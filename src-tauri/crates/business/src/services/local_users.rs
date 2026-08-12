use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{dto::LocalApiPermissionDefinitionDto, svc_ctx::SvcCtx};

const PASSWORD_HASH_ROUNDS: u32 = 100_000;
pub const LOCAL_USER_AVATARS: &[&str] = &[
    "🙂", "😎", "🦊", "🐼", "🐯", "🐙", "🦉", "🐳", "🌙", "⭐", "🌿", "🚀",
];

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LocalUser {
    pub uuid: Uuid,
    pub nickname: String,
    pub avatar: String,
    pub has_password: bool,
    pub current_workspace_uuid: Option<Uuid>,
    pub current_team_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalUserRequest {
    pub nickname: String,
    pub avatar: String,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginLocalUserRequest {
    pub user_uuid: Uuid,
    pub password: Option<String>,
}

pub async fn list_local_users(context: &SvcCtx) -> Result<Vec<LocalUser>, String> {
    sqlx::query_as::<_, LocalUser>(
        r#"
        SELECT
            u.uuid,
            COALESCE(NULLIF(ui.nickname, ''), 'Local User') AS nickname,
            lua.avatar,
            (lua.password_hash IS NOT NULL) AS has_password,
            ui.current_workspace_uuid,
            ui.current_team_uuid
        FROM users u
        JOIN user_infos ui ON ui.user_uuid = u.uuid
        JOIN local_user_auth lua ON lua.user_uuid = u.uuid
        WHERE u.deleted_at IS NULL AND ui.deleted_at IS NULL
        ORDER BY lua.created_at, ui.id
        "#,
    )
    .fetch_all(&context.db)
    .await
    .map_err(|error| error.to_string())
}

pub async fn current_local_user(context: &SvcCtx) -> Result<Option<LocalUser>, String> {
    let Some(user_uuid) = context.current_user_uuid() else {
        return Ok(None);
    };
    fetch_local_user(context, user_uuid).await
}

pub async fn create_local_user(
    context: &SvcCtx,
    request: &CreateLocalUserRequest,
) -> Result<LocalUser, String> {
    let nickname = validate_nickname(&request.nickname)?;
    let avatar = validate_avatar(&request.avatar)?;
    let password = normalize_password(request.password.as_deref());
    let password_credentials = password.map(create_password_credentials);
    let permission_definitions =
        crate::models::local_api::fetch_permission_definitions(&context.db)
            .await
            .map_err(|error| error.to_string())?;
    let mut transaction = context.db.begin().await.map_err(|error| error.to_string())?;
    let user_uuid = Uuid::new_v4();
    let user_id = format!("LOCAL-{}", user_uuid.simple());

    sqlx::query("INSERT INTO users (uuid, id) VALUES ($1, $2)")
        .bind(user_uuid)
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;

    // `email` and `password` are legacy columns retained while the copied business schema is
    // being reduced. They are not local-account attributes and are never exposed to the UI.
    sqlx::query(
        "INSERT INTO user_infos (user_uuid, nickname, email, password, avatar_hash, status) \
         VALUES ($1, $2, $3, '', $4, 'active')",
    )
    .bind(user_uuid)
    .bind(&nickname)
    .bind(format!("local-{user_uuid}@simprint.invalid"))
    .bind(&avatar)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    let (password_salt, password_hash) = password_credentials
        .map(|(salt, hash)| (Some(salt), Some(hash)))
        .unwrap_or((None, None));
    sqlx::query(
        "INSERT INTO local_user_auth (user_uuid, avatar, password_salt, password_hash) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_uuid)
    .bind(&avatar)
    .bind(password_salt)
    .bind(password_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    sqlx::query(
        "INSERT INTO user_preferences (user_uuid, theme, language, notifications_enabled) \
         VALUES ($1, 'system', 'zh-CN', TRUE)",
    )
    .bind(user_uuid)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    sqlx::query(
        "INSERT INTO user_local_api_settings \
         (user_uuid, enabled, port, remote_access, cors_origins) \
         VALUES ($1, FALSE, 8080, FALSE, '[]')",
    )
    .bind(user_uuid)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    initialize_local_api_key(&mut transaction, user_uuid, &permission_definitions).await?;

    let workspace_uuid: Uuid = sqlx::query_scalar(
        "INSERT INTO workspaces (name, owner_uuid, workspace_type) \
         VALUES ($1, $2, 'personal') RETURNING uuid",
    )
    .bind(format!("{nickname} 的工作空间"))
    .bind(user_uuid)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    let quota = &context.workspace_quota.default;
    sqlx::query(
        "INSERT INTO workspace_quotas \
         (workspace_uuid, max_environments, max_team_members, max_proxies, max_rpa_tasks) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(workspace_uuid)
    .bind(quota.max_environments)
    .bind(quota.max_team_members)
    .bind(quota.max_proxies)
    .bind(quota.max_rpa_tasks)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    let team_uuid: Uuid = sqlx::query_scalar(
        "INSERT INTO teams (workspace_uuid, name, description, owner_uuid) \
         VALUES ($1, $2, '默认团队', $3) RETURNING uuid",
    )
    .bind(workspace_uuid)
    .bind(format!("{nickname} 的团队"))
    .bind(user_uuid)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    sqlx::query(
        "INSERT INTO team_members (team_uuid, workspace_uuid, user_uuid, role, status) \
         VALUES ($1, $2, $3, 'owner', 'active')",
    )
    .bind(team_uuid)
    .bind(workspace_uuid)
    .bind(user_uuid)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    sqlx::query(
        "UPDATE user_infos SET current_workspace_uuid = $1, current_team_uuid = $2 \
         WHERE user_uuid = $3",
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(user_uuid)
    .execute(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    transaction.commit().await.map_err(|error| error.to_string())?;
    fetch_local_user(context, user_uuid)
        .await?
        .ok_or_else(|| "本地用户创建后无法读取".to_string())
}

pub async fn authenticate_local_user(
    context: &SvcCtx,
    request: &LoginLocalUserRequest,
) -> Result<LocalUser, String> {
    verify_local_user_password(context, request.user_uuid, request.password.as_deref()).await?;

    let user = fetch_local_user(context, request.user_uuid)
        .await?
        .ok_or_else(|| "本地用户不存在".to_string())?;
    context.authenticate_user(user.uuid);
    Ok(user)
}

pub async fn verify_local_user_password(
    context: &SvcCtx,
    user_uuid: Uuid,
    password: Option<&str>,
) -> Result<(), String> {
    let credentials = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT password_salt, password_hash FROM local_user_auth WHERE user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_optional(&context.db)
    .await
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "本地用户不存在".to_string())?;

    match credentials {
        (None, None) => {}
        (Some(salt), Some(expected_hash)) => {
            let password = password.unwrap_or_default();
            let actual_hash = derive_password_hash(password, &salt);
            if !constant_time_equal(actual_hash.as_bytes(), expected_hash.as_bytes()) {
                return Err("密码不正确".to_string());
            }
        }
        _ => return Err("本地用户密码状态无效".to_string()),
    }
    Ok(())
}

async fn fetch_local_user(context: &SvcCtx, user_uuid: Uuid) -> Result<Option<LocalUser>, String> {
    sqlx::query_as::<_, LocalUser>(
        r#"
        SELECT
            u.uuid,
            COALESCE(NULLIF(ui.nickname, ''), 'Local User') AS nickname,
            lua.avatar,
            (lua.password_hash IS NOT NULL) AS has_password,
            ui.current_workspace_uuid,
            ui.current_team_uuid
        FROM users u
        JOIN user_infos ui ON ui.user_uuid = u.uuid
        JOIN local_user_auth lua ON lua.user_uuid = u.uuid
        WHERE u.uuid = $1 AND u.deleted_at IS NULL AND ui.deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(&context.db)
    .await
    .map_err(|error| error.to_string())
}

async fn initialize_local_api_key(
    transaction: &mut sqlx::Transaction<'_, crate::database::Db>,
    user_uuid: Uuid,
    definitions: &[LocalApiPermissionDefinitionDto],
) -> Result<(), String> {
    let api_key = format!("sk_local_{}", Uuid::new_v4().simple());
    let key_hash = crate::models::local_api::hash_api_key(&api_key);
    let key_id: i32 = sqlx::query_scalar(
        "INSERT INTO user_local_api_keys \
         (user_uuid, key_prefix, key_hash, api_key, daily_limit) \
         VALUES ($1, $2, $3, $4, 1000) RETURNING id",
    )
    .bind(user_uuid)
    .bind(api_key.chars().take(16).collect::<String>())
    .bind(key_hash)
    .bind(api_key)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| error.to_string())?;

    for definition in definitions {
        sqlx::query(
            "INSERT INTO user_local_api_key_permissions \
             (api_key_id, permission_code, is_enabled, rate_limit_per_minute, rate_limit_per_hour) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(key_id)
        .bind(&definition.permission_code)
        .bind(definition.default_enabled)
        .bind(definition.default_rate_limit_per_minute)
        .bind(definition.default_rate_limit_per_hour)
        .execute(&mut **transaction)
        .await
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn validate_nickname(value: &str) -> Result<String, String> {
    let nickname = value.trim();
    if nickname.is_empty() {
        return Err("请输入用户昵称".to_string());
    }
    if nickname.chars().count() > 64 {
        return Err("用户昵称不能超过 64 个字符".to_string());
    }
    Ok(nickname.to_string())
}

fn validate_avatar(value: &str) -> Result<String, String> {
    if LOCAL_USER_AVATARS.contains(&value) {
        Ok(value.to_string())
    } else {
        Err("请选择有效的本地用户图标".to_string())
    }
}

fn normalize_password(value: Option<&str>) -> Option<&str> {
    value.filter(|password| !password.is_empty())
}

fn create_password_credentials(password: &str) -> (String, String) {
    let salt = Uuid::new_v4().simple().to_string();
    let hash = derive_password_hash(password, &salt);
    (salt, hash)
}

fn derive_password_hash(password: &str, salt: &str) -> String {
    let mut state = Sha256::digest([salt.as_bytes(), password.as_bytes()].concat()).to_vec();
    for round in 0..PASSWORD_HASH_ROUNDS {
        let mut hasher = Sha256::new();
        hasher.update(&state);
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        hasher.update(round.to_le_bytes());
        state = hasher.finalize().to_vec();
    }
    hex::encode(state)
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter().zip(right).fold(0_u8, |difference, (left, right)| {
        difference | (left ^ right)
    }) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::DatabaseConfig;

    #[tokio::test]
    async fn creates_and_authenticates_password_and_passwordless_users() {
        let mut config = DatabaseConfig::embedded("sqlite::memory:");
        config.max_connections = 1;
        config.min_connections = 1;
        let context = SvcCtx::new(&config).await.unwrap();

        let passwordless = create_local_user(
            &context,
            &CreateLocalUserRequest {
                nickname: "Alice".to_string(),
                avatar: "🦊".to_string(),
                password: None,
            },
        )
        .await
        .unwrap();
        assert!(!passwordless.has_password);
        authenticate_local_user(
            &context,
            &LoginLocalUserRequest {
                user_uuid: passwordless.uuid,
                password: None,
            },
        )
        .await
        .unwrap();

        let protected = create_local_user(
            &context,
            &CreateLocalUserRequest {
                nickname: "Bob".to_string(),
                avatar: "🚀".to_string(),
                password: Some("secret".to_string()),
            },
        )
        .await
        .unwrap();
        assert!(protected.has_password);
        assert!(
            authenticate_local_user(
                &context,
                &LoginLocalUserRequest {
                    user_uuid: protected.uuid,
                    password: Some("wrong".to_string()),
                },
            )
            .await
            .is_err()
        );
        authenticate_local_user(
            &context,
            &LoginLocalUserRequest {
                user_uuid: protected.uuid,
                password: Some("secret".to_string()),
            },
        )
        .await
        .unwrap();
        assert_eq!(context.current_user_uuid(), Some(protected.uuid));
    }
}
