use sqlx::Error;
use uuid::Uuid;

use crate::database::{Db, Pool};

use crate::dto::{LocalApiPermissionDefinitionDto, UserDto, UserInfoDto};
use crate::entitys::{RegisterRequest, UpdateUserRequest};

/// 插入用户基础信息
pub async fn insert_user(pool: &Pool<Db>, user_id: String) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar("INSERT INTO users (id) VALUES ($1) RETURNING uuid;")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(uuid)
}

/// 插入用户详细信息
pub async fn insert_user_info(
    pool: &Pool<Db>,
    user_uuid: Uuid,
    payload: &RegisterRequest,
    password_hash: &str,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO user_infos (user_uuid, email, password, nickname, status)
        VALUES ($1, $2, $3, $4, 'active')
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(&payload.email)
    .bind(password_hash)
    .bind(&payload.nickname)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 使用事务创建用户（users + user_infos + 初始化数据）
///
/// 初始化数据包括：
/// - user_preferences: 用户偏好设置
/// - teams: 个人团队（每个用户自动创建一个团队）
pub async fn create_user_with_info(
    pool: &Pool<Db>,
    user_id: String,
    payload: &RegisterRequest,
    password_hash: &str,
    quota: &crate::utils::WorkspaceQuotaValues,
) -> Result<Uuid, Error> {
    let local_api_permission_definitions =
        crate::models::local_api::fetch_permission_definitions(pool).await?;

    let mut tx = pool.begin().await?;

    // 1. 创建用户基础记录
    let user_uuid: Uuid = sqlx::query_scalar("INSERT INTO users (id) VALUES ($1) RETURNING uuid;")
        .bind(&user_id)
        .fetch_one(&mut *tx)
        .await?;

    // 2. 创建用户详细信息
    sqlx::query(
        r#"
        INSERT INTO user_infos (user_uuid, email, password, nickname, status)
        VALUES ($1, $2, $3, $4, 'active');
        "#,
    )
    .bind(user_uuid)
    .bind(&payload.email)
    .bind(password_hash)
    .bind(&payload.nickname)
    .execute(&mut *tx)
    .await?;

    // 3. 初始化用户偏好设置
    sqlx::query(
        r#"
        INSERT INTO user_preferences (user_uuid, theme, language, notifications_enabled)
        VALUES ($1, 'system', 'zh-CN', TRUE);
        "#,
    )
    .bind(user_uuid)
    .execute(&mut *tx)
    .await?;

    // 4. 初始化本地 API 配置
    sqlx::query(
        r#"
        INSERT INTO user_local_api_settings (user_uuid, enabled, port, remote_access, cors_origins)
        VALUES ($1, FALSE, 8080, FALSE, '[]');
        "#,
    )
    .bind(user_uuid)
    .execute(&mut *tx)
    .await?;

    // 5. 初始化本地 API 密钥
    let local_api_key = generate_local_api_key();
    let local_api_key_hash = crate::models::local_api::hash_api_key(&local_api_key);
    let local_api_key_prefix = local_api_key.chars().take(16).collect::<String>();
    let local_api_key_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO user_local_api_keys (user_uuid, key_prefix, key_hash, api_key, daily_limit)
        VALUES ($1, $2, $3, $4, 1000)
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(&local_api_key_prefix)
    .bind(&local_api_key_hash)
    .bind(&local_api_key)
    .fetch_one(&mut *tx)
    .await?;

    // 6. 初始化本地 API 权限记录
    insert_local_api_permissions(&mut tx, local_api_key_id, &local_api_permission_definitions)
        .await?;

    // 7. 创建个人团队（每个用户都应该有一个团队）
    // 团队名称使用用户昵称，如果没有昵称则使用邮箱前缀
    let team_name =
        payload.nickname.as_ref().map(|n| format!("{} 的团队", n)).unwrap_or_else(|| {
            format!(
                "{} 的团队",
                payload.email.split('@').next().unwrap_or("用户")
            )
        });

    // 8. 创建个人工作空间
    let workspace_name = format!(
        "{} 的工作空间",
        payload.nickname.as_deref().unwrap_or("用户")
    );
    let workspace_uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workspaces (name, owner_uuid, workspace_type)
        VALUES ($1, $2, 'personal')
        RETURNING uuid;
        "#,
    )
    .bind(&workspace_name)
    .bind(user_uuid)
    .fetch_one(&mut *tx)
    .await?;

    // 9. 创建工作空间配额（使用传入的配额配置）
    sqlx::query(
        r#"
        INSERT INTO workspace_quotas (workspace_uuid, max_environments, max_team_members, max_proxies, max_rpa_tasks)
        VALUES ($1, $2, $3, $4, $5);
        "#,
    )
    .bind(workspace_uuid)
    .bind(quota.max_environments)
    .bind(quota.max_team_members)
    .bind(quota.max_proxies)
    .bind(quota.max_rpa_tasks)
    .execute(&mut *tx)
    .await?;

    // 10. 创建个人团队（每个用户自动创建一个团队）
    let team_uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO teams (workspace_uuid, name, description, owner_uuid)
        VALUES ($1, $2, $3, $4)
        RETURNING uuid;
        "#,
    )
    .bind(workspace_uuid)
    .bind(&team_name)
    .bind(Some("个人团队"))
    .bind(user_uuid)
    .fetch_one(&mut *tx)
    .await?;

    // 11. 添加用户为团队成员（owner 角色）
    sqlx::query(
        r#"
        INSERT INTO team_members (team_uuid, workspace_uuid, user_uuid, role, status)
        VALUES ($1, $2, $3, 'owner', 'active');
        "#,
    )
    .bind(team_uuid)
    .bind(workspace_uuid)
    .bind(user_uuid)
    .execute(&mut *tx)
    .await?;

    // 12. 设置为用户当前团队和工作空间
    sqlx::query(
        r#"
        UPDATE user_infos SET current_team_uuid = $1, current_workspace_uuid = $2 WHERE user_uuid = $3;
        "#,
    )
    .bind(team_uuid)
    .bind(workspace_uuid)
    .bind(user_uuid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(user_uuid)
}

async fn insert_local_api_permissions(
    tx: &mut sqlx::Transaction<'_, Db>,
    api_key_id: i32,
    definitions: &[LocalApiPermissionDefinitionDto],
) -> Result<(), Error> {
    for definition in definitions {
        sqlx::query(
            r#"
            INSERT INTO user_local_api_key_permissions (
                api_key_id, permission_code, is_enabled, rate_limit_per_minute, rate_limit_per_hour
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (api_key_id, permission_code) DO NOTHING;
            "#,
        )
        .bind(api_key_id)
        .bind(definition.permission_code.as_str())
        .bind(definition.default_enabled)
        .bind(definition.default_rate_limit_per_minute)
        .bind(definition.default_rate_limit_per_hour)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

fn generate_local_api_key() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    format!("sk_local_{}", raw)
}

/// 根据 UUID 查询用户
pub async fn fetch_user_by_uuid(
    pool: &Pool<Db>,
    user_uuid: Uuid,
) -> Result<Option<UserDto>, Error> {
    let rec = sqlx::query_as::<_, UserDto>(
        r#"
        SELECT uuid, id, created_at, updated_at, deleted_at
        FROM users
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询用户详细信息
pub async fn fetch_user_info_by_uuid(
    pool: &Pool<Db>,
    user_uuid: Uuid,
) -> Result<Option<UserInfoDto>, Error> {
    let rec = sqlx::query_as::<_, UserInfoDto>(
        r#"
        SELECT id, user_uuid, nickname, email, phone, password, avatar_hash, status,
               current_team_uuid, current_workspace_uuid, created_at, updated_at, deleted_at
        FROM user_infos
        WHERE user_uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 根据邮箱查询用户详细信息
pub async fn fetch_user_info_by_email(
    pool: &Pool<Db>,
    email: &str,
) -> Result<Option<UserInfoDto>, Error> {
    let rec = sqlx::query_as::<_, UserInfoDto>(
        r#"
        SELECT id, user_uuid, nickname, email, phone, password, avatar_hash, status,
               current_team_uuid, current_workspace_uuid, created_at, updated_at, deleted_at
        FROM user_infos
        WHERE email = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新用户信息
pub async fn update_user_info(
    pool: &Pool<Db>,
    user_uuid: Uuid,
    payload: &UpdateUserRequest,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_infos
        SET nickname = COALESCE($1, nickname),
            phone = COALESCE($2, phone),
            email = COALESCE($3, email),
            updated_at = CURRENT_TIMESTAMP
        WHERE user_uuid = $4 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.nickname.as_deref())
    .bind(payload.phone.as_deref())
    .bind(payload.email.as_deref())
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新密码
pub async fn update_password(
    pool: &Pool<Db>,
    user_uuid: Uuid,
    password_hash: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_infos
        SET password = $1
        WHERE user_uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(password_hash)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 设置用户当前工作空间
pub async fn fetch_user_current_workspace(
    pool: &Pool<Db>,
    user_uuid: Uuid,
) -> Result<Option<Uuid>, Error> {
    let rec: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT current_workspace_uuid FROM user_infos
        WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

pub async fn set_user_current_workspace(
    pool: &Pool<Db>,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_infos SET current_workspace_uuid = $1 WHERE user_uuid = $2
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_user_current_workspace_and_team(
    pool: &Pool<Db>,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_infos
        SET current_workspace_uuid = $1, current_team_uuid = $2
        WHERE user_uuid = $3
        "#,
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}
