use crate::svc_ctx::SvcCtx;
use redis::AsyncCommands;
use uuid::Uuid;

pub(crate) const REGISTER_CODE_CACHE_KEY: &str = "verification:code:register:send";
pub(crate) const RESET_PASSWORD_CODE_CACHE_KEY: &str = "verification:code:reset_password:send";
pub(crate) const CODE_EXPIRATION: u64 = 60 * 5; // 5 分钟过期
pub(crate) const USER_PUBLIC_KEY_CACHE_KEY: &str = "user:public_key";
pub(crate) const PUBLIC_KEY_EXPIRATION: u64 = 60 * 60 * 24 * 7; // 7 天过期（与 refresh token 一致）

/// 设置用户注册的验证码
pub async fn set_register_code(
    svc_ctx: &SvcCtx,
    email: &str,
    code: &str,
) -> Result<(), anyhow::Error> {
    let key = format!("{}:{}", REGISTER_CODE_CACHE_KEY, email);
    let _: () = svc_ctx.redis.clone().set_ex(key, code, CODE_EXPIRATION).await?;
    Ok(())
}

/// 获取用户注册的验证码
pub async fn get_register_code(
    svc_ctx: &SvcCtx,
    email: &str,
) -> Result<Option<String>, anyhow::Error> {
    let key = format!("{}:{}", REGISTER_CODE_CACHE_KEY, email);
    let result: Option<String> = svc_ctx.redis.clone().get(key).await?;
    Ok(result)
}

/// 设置重置密码的验证码
pub async fn set_reset_password_code(
    svc_ctx: &SvcCtx,
    email: &str,
    code: &str,
) -> Result<(), anyhow::Error> {
    let key = format!("{}:{}", RESET_PASSWORD_CODE_CACHE_KEY, email);
    let _: () = svc_ctx.redis.clone().set_ex(key, code, CODE_EXPIRATION).await?;
    Ok(())
}

/// 获取重置密码的验证码
pub async fn get_reset_password_code(
    svc_ctx: &SvcCtx,
    email: &str,
) -> Result<Option<String>, anyhow::Error> {
    let key = format!("{}:{}", RESET_PASSWORD_CODE_CACHE_KEY, email);
    let result: Option<String> = svc_ctx.redis.clone().get(key).await?;
    Ok(result)
}

/// 设置用户公钥
pub async fn set_user_public_key(
    svc_ctx: &SvcCtx,
    user_uuid: &Uuid,
    public_key: &str,
) -> Result<(), anyhow::Error> {
    let key = format!("{}:{}", USER_PUBLIC_KEY_CACHE_KEY, user_uuid);
    let _: () = svc_ctx.redis.clone().set_ex(key, public_key, PUBLIC_KEY_EXPIRATION).await?;
    Ok(())
}

/// 获取用户公钥
pub async fn get_user_public_key(
    svc_ctx: &SvcCtx,
    user_uuid: &Uuid,
) -> Result<Option<String>, anyhow::Error> {
    let key = format!("{}:{}", USER_PUBLIC_KEY_CACHE_KEY, user_uuid);
    let result: Option<String> = svc_ctx.redis.clone().get(key).await?;
    Ok(result)
}
