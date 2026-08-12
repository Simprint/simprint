use uuid::Uuid;

use crate::caches::{
    get_register_code, get_reset_password_code, get_user_public_key, set_register_code,
    set_reset_password_code, set_user_public_key,
};
use crate::entitys::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, ResetPasswordRequest,
    UpdatePasswordRequest, UpdateUserRequest, UserResponse, VerifyPasswordRequest,
    VerifyPasswordResponse,
};
use crate::models::{
    billing,
    user::{
        create_user_with_info, fetch_user_by_uuid, fetch_user_info_by_email,
        fetch_user_info_by_uuid, update_password, update_user_info,
    },
};
use crate::services::messages;
use crate::svc_ctx::SvcCtx;
use crate::utils::{
    encryption_password, generate_token, random_six_number_code, send_email, verify_password,
};

/// 通用登录逻辑（内部辅助函数）
///
/// 处理登录、注册、记住密码登录的共同逻辑：
/// - 生成 Token
/// - 保存公钥到缓存
/// - 获取用户信息
async fn common_login_logic(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    public_key: Option<&String>,
) -> Result<(String, String, Option<UserResponse>), anyhow::Error> {
    let secret = svc_ctx.config.app.secret.as_bytes();

    // 1. 生成 Token
    let access_token = generate_token(&user_uuid.to_string(), 60 * 60 * 2, secret)?; // 2 小时
    let refresh_token = generate_token(&user_uuid.to_string(), 60 * 60 * 24 * 7, secret)?; // 7 天

    // 2. 如果提供了公钥，保存到缓存
    if let Some(ref public_key) = public_key {
        if !public_key.is_empty() {
            let _ = set_user_public_key(svc_ctx, &user_uuid, public_key).await;
            // 保存失败不影响登录流程，只记录错误
        }
    }

    // 3. 获取用户信息
    let user_info = get_current_user_service(svc_ctx, user_uuid).await.ok();

    Ok((access_token, refresh_token, user_info))
}

/// 用户注册服务
///
/// # Arguments
/// * `svc_ctx` - 服务上下文
/// * `payload` - 注册请求数据
///
/// # Returns
/// 返回注册响应
pub async fn register_service(
    svc_ctx: &SvcCtx,
    payload: &RegisterRequest,
) -> Result<RegisterResponse, anyhow::Error> {
    let pool = &svc_ctx.db;
    // 1. 验证验证码
    let cached_code = get_register_code(svc_ctx, &payload.email).await?;
    if cached_code != Some(payload.code.clone()) {
        return Err(anyhow::anyhow!("验证码错误或已过期"));
    }

    // 2. 检查邮箱是否已存在
    let existing_user = fetch_user_info_by_email(pool, &payload.email).await?;
    if existing_user.is_some() {
        return Err(anyhow::anyhow!("邮箱已被注册"));
    }

    // 3. 加密密码
    let password_hash = encryption_password(&payload.password)?;

    // 4. 生成用户 ID（使用 UUID 的前8位作为用户 ID）
    let user_id = format!("USER{}", Uuid::new_v4().to_string()[..8].to_uppercase());

    // 5. 创建用户（传入默认配额配置）
    let quota = &svc_ctx.config.workspace_quota.default;
    let user_uuid = create_user_with_info(pool, user_id, payload, &password_hash, quota).await?;

    // 5.1. 自动发放新用户优惠券（失败不影响注册流程）
    let _ = issue_welcome_coupons(svc_ctx, user_uuid).await;

    // 6. 执行通用登录逻辑
    let (access_token, refresh_token, user_info) =
        common_login_logic(svc_ctx, user_uuid, payload.public_secret_key.as_ref()).await?;

    // 7. 发送欢迎系统通知
    let _ = messages::create_message_service(
        svc_ctx,
        None, // 系统消息，无发送者
        &crate::entitys::CreateMessageRequest {
            message_type: "system_notification".to_string(),
            title: "欢迎加入".to_string(),
            content: Some("感谢您的注册！我们很高兴您能加入我们。祝您使用愉快！".to_string()),
            recipient_uuids: vec![user_uuid],
            recipient_type: "single".to_string(),
            related_type: None,
            related_uuid: None,
            priority: Some("normal".to_string()),
            metadata: None,
        },
    )
    .await;
    // 注意：消息创建失败不影响注册流程，使用 _ 忽略错误

    Ok(RegisterResponse {
        access_token,
        refresh_token,
        user_info,
    })
}

/// 自动发放新用户优惠券
///
/// 在用户注册时自动发放欢迎优惠券，失败不影响注册流程
async fn issue_welcome_coupons(svc_ctx: &SvcCtx, user_uuid: Uuid) -> Result<(), anyhow::Error> {
    // 新用户优惠券代码列表（可根据需要配置）
    let welcome_coupon_codes = vec!["WELCOME10", "FIRST10"];

    for coupon_code in welcome_coupon_codes {
        // 查询优惠券
        if let Some(coupon) = billing::fetch_coupon_by_code(&svc_ctx.db, coupon_code).await? {
            // 发放优惠券给用户（继承优惠券的过期时间）
            let _ = billing::insert_user_coupon(
                &svc_ctx.db,
                user_uuid,
                coupon.uuid,
                coupon.valid_until,
            )
            .await;
            // 注意：如果用户已拥有该优惠券（重复发放），忽略错误
        }
    }

    Ok(())
}

/// 用户登录服务（统一处理两种登录方式）
///
/// # Arguments
/// * `svc_ctx` - 服务上下文
/// * `payload` - 登录请求数据
///
/// # Returns
/// 返回登录响应
pub async fn login_service(
    svc_ctx: &SvcCtx,
    payload: LoginRequest,
) -> Result<LoginResponse, anyhow::Error> {
    let pool = &svc_ctx.db;
    let (user_uuid, public_secret_key) = match payload {
        LoginRequest::Basic(data) => {
            // 基本登录：验证邮箱和密码
            // 1. 查询用户
            let user_info = fetch_user_info_by_email(pool, &data.email)
                .await?
                .ok_or_else(|| anyhow::anyhow!("账号不存在"))?;

            // 2. 验证密码
            if !verify_password(&data.password, &user_info.password) {
                return Err(anyhow::anyhow!("密码错误"));
            }

            // 3. 检查用户状态
            if user_info.status != "active" {
                return Err(anyhow::anyhow!("用户已被禁用"));
            }

            (user_info.user_uuid, data.public_secret_key)
        }
        LoginRequest::Remember(data) => {
            // 记住密码登录：验证 refresh_token
            // 1. 验证 refresh_token
            let secret = svc_ctx.config.app.secret.as_bytes();
            let user_uuid_str = verify_token_service(&data.refresh_token, secret)?;
            let user_uuid = Uuid::parse_str(&user_uuid_str)?;

            // 2. 查询用户信息
            let user_info = fetch_user_info_by_uuid(pool, user_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("账号不存在"))?;

            // 3. 验证邮箱是否匹配
            if user_info.email != data.email {
                return Err(anyhow::anyhow!("邮箱不匹配"));
            }

            // 4. 检查用户状态
            if user_info.status != "active" {
                return Err(anyhow::anyhow!("用户已被禁用"));
            }

            (user_uuid, data.public_secret_key)
        }
    };

    // 执行通用登录逻辑
    let (access_token, refresh_token, user_response) =
        common_login_logic(svc_ctx, user_uuid, public_secret_key.as_ref()).await?;

    Ok(LoginResponse {
        access_token,
        refresh_token,
        user_info: user_response,
    })
}

/// 验证 Token 服务
///
/// # Arguments
/// * `token` - 要验证的 token
/// * `secret` - JWT 密钥
///
/// # Returns
/// 返回用户 UUID
pub fn verify_token_service(token: &str, secret: &[u8]) -> Result<String, anyhow::Error> {
    crate::utils::verify_token(token, secret)
}

/// 刷新 Token 服务
///
/// # Arguments
/// * `pool` - 数据库连接池
/// * `svc_ctx` - 服务上下文
/// * `refresh_token` - 刷新令牌
///
/// # Returns
/// 返回登录响应
pub async fn refresh_token_service(
    svc_ctx: &SvcCtx,
    refresh_token: &str,
) -> Result<LoginResponse, anyhow::Error> {
    let pool = &svc_ctx.db;
    // 1. 验证 refresh_token
    let secret = svc_ctx.config.app.secret.as_bytes();
    let user_uuid_str = verify_token_service(refresh_token, secret)?;

    // 2. 验证用户是否存在
    let user_uuid = Uuid::parse_str(&user_uuid_str)?;
    let user = fetch_user_by_uuid(pool, user_uuid).await?;
    if user.is_none() {
        return Err(anyhow::anyhow!("用户不存在"));
    }

    // 3. 生成新的 Token
    let access_token = generate_token(&user_uuid_str, 60 * 60 * 2, secret)?; // 2 小时
    let new_refresh_token = generate_token(&user_uuid_str, 60 * 60 * 24 * 7, secret)?; // 7 天

    Ok(LoginResponse {
        access_token,
        refresh_token: new_refresh_token,
        user_info: None,
    })
}

/// 获取当前用户信息服务
///
/// # Arguments
/// * `svc_ctx` - 服务上下文
/// * `user_uuid` - 用户 UUID
///
/// # Returns
/// 返回用户信息响应（包含当前团队信息）
pub async fn get_current_user_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<UserResponse, anyhow::Error> {
    let pool = &svc_ctx.db;
    let user = fetch_user_by_uuid(pool, user_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

    let user_info = fetch_user_info_by_uuid(pool, user_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户信息不存在"))?;

    // 获取当前团队详细信息
    let current_team = if let Some(team_uuid) = user_info.current_team_uuid {
        // 查询团队详细信息
        crate::models::teams::fetch_team_by_uuid(pool, team_uuid).await.ok().flatten()
    } else {
        None
    };

    // 获取当前工作空间详细信息
    let current_workspace = if let Some(workspace_uuid) = user_info.current_workspace_uuid {
        // 查询工作空间详细信息
        crate::models::workspaces::fetch_workspace_by_uuid(pool, workspace_uuid)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    let avatar_url = user_info.avatar_hash.as_ref().map(|hash| {
        crate::utils::storage::get_objects::get_avatar_url(
            &svc_ctx.config.storage.public_base_url,
            &svc_ctx.config.storage.avatar_root,
            hash,
        )
    });

    Ok(UserResponse {
        uuid: user.uuid,
        id: user.id,
        nickname: user_info.nickname,
        email: user_info.email,
        phone: user_info.phone,
        avatar_hash: user_info.avatar_hash,
        avatar_url,
        status: user_info.status,
        created_at: user.created_at,
        updated_at: user.updated_at,
        current_team,
        current_workspace,
    })
}

/// 更新用户信息服务
///
/// # Arguments
/// * `pool` - 数据库连接池
/// * `user_uuid` - 用户 UUID
/// * `payload` - 更新请求数据
pub async fn update_user_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdateUserRequest,
) -> Result<(), anyhow::Error> {
    let pool = &svc_ctx.db;
    update_user_info(pool, user_uuid, payload).await?;
    Ok(())
}

/// 修改密码服务
///
/// # Arguments
/// * `pool` - 数据库连接池
/// * `user_uuid` - 用户 UUID
/// * `payload` - 修改密码请求数据
pub async fn update_password_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdatePasswordRequest,
) -> Result<(), anyhow::Error> {
    let pool = &svc_ctx.db;
    // 1. 查询用户
    let user_info = fetch_user_info_by_uuid(pool, user_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

    // 2. 验证旧密码
    if !verify_password(&payload.old_password, &user_info.password) {
        return Err(anyhow::anyhow!("原密码错误"));
    }

    // 3. 加密新密码
    let password_hash = encryption_password(&payload.new_password)?;

    // 4. 更新密码
    update_password(pool, user_uuid, &password_hash).await?;

    Ok(())
}

/// 校验当前用户密码
pub async fn verify_password_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &VerifyPasswordRequest,
) -> Result<VerifyPasswordResponse, anyhow::Error> {
    let user_info = fetch_user_info_by_uuid(&svc_ctx.db, user_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

    if user_info.status != "active" {
        return Err(anyhow::anyhow!("用户已被禁用"));
    }

    Ok(VerifyPasswordResponse {
        valid: verify_password(&payload.password, &user_info.password),
    })
}

/// 重置密码服务
///
/// # Arguments
/// * `pool` - 数据库连接池
/// * `svc_ctx` - 服务上下文
/// * `payload` - 重置密码请求数据
pub async fn reset_password_service(
    svc_ctx: &SvcCtx,
    payload: &ResetPasswordRequest,
) -> Result<(), anyhow::Error> {
    let pool = &svc_ctx.db;
    // 1. 验证验证码
    let cached_code = get_reset_password_code(svc_ctx, &payload.email).await?;
    if cached_code != Some(payload.code.clone()) {
        return Err(anyhow::anyhow!("验证码错误或已过期"));
    }

    // 2. 查询用户
    let user_info = fetch_user_info_by_email(pool, &payload.email)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

    // 3. 加密新密码
    let password_hash = encryption_password(&payload.new_password)?;

    // 4. 更新密码
    update_password(pool, user_info.user_uuid, &password_hash).await?;

    Ok(())
}

/// 发送验证码服务
///
/// # Arguments
/// * `svc_ctx` - 服务上下文
/// * `email` - 邮箱地址
/// * `code_type` - 验证码类型（register 或 reset_password）
pub async fn send_verification_code_service(
    svc_ctx: &SvcCtx,
    email: &str,
    code_type: &str,
) -> Result<(), anyhow::Error> {
    if code_type == "reset_password" {
        let user_exists = fetch_user_info_by_email(&svc_ctx.db, email).await?.is_some();
        if !user_exists {
            return Err(anyhow::anyhow!("用户不存在"));
        }
    }

    let smtp_config = svc_ctx
        .config
        .clone()
        .smtp
        .ok_or_else(|| anyhow::anyhow!("邮箱配置不存在"))?;

    // 1. 生成验证码
    let code = random_six_number_code();

    // 2. 存储验证码到 Redis
    match code_type {
        "register" => set_register_code(svc_ctx, email, &code).await?,
        "reset_password" => set_reset_password_code(svc_ctx, email, &code).await?,
        _ => return Err(anyhow::anyhow!("无效的验证码类型")),
    }

    // 3. 发送邮件
    let title = match code_type {
        "register" => "注册验证码",
        "reset_password" => "重置密码验证码",
        _ => "验证码",
    };
    let body = format!("您的验证码是：{}，有效期 5 分钟。", code);

    send_email(
        &smtp_config.smtp_username,
        &smtp_config.smtp_password,
        &smtp_config.smtp_server,
        email,
        title,
        &body,
    )?;

    Ok(())
}

/// 获取用户公钥
///
/// # Arguments
/// * `svc_ctx` - 服务上下文
/// * `user_uuid` - 用户对应的UUID
///
/// # Returns
/// 返回用户公钥，如果获取成功则返回公钥，否则返回错误
pub async fn get_user_public_key_service(
    svc_ctx: &SvcCtx,
    user_uuid: &Uuid,
) -> Result<String, anyhow::Error> {
    let public_key = get_user_public_key(svc_ctx, user_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("用户公钥不存在"))?;
    Ok(public_key)
}
