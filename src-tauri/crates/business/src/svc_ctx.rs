use crate::{
    database::{self, DbPool},
    entitys::CreateWorkspaceRequest,
    utils::{DatabaseConfig, WorkspaceQuotaConfig},
};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Shared resources used by handlers and services.
#[derive(Clone)]
pub struct SvcCtx {
    pub db: DbPool,
    pub workspace_quota: WorkspaceQuotaConfig,
    pub local_user_uuid: Uuid,
    session_user_uuid: Arc<RwLock<Option<Uuid>>>,
}

impl SvcCtx {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, anyhow::Error> {
        let db = Self::create_db(config).await?;
        database::migrate(&db).await?;
        crate::services::browser_kernels::import_default_catalog(&db)
            .await
            .map_err(anyhow::Error::msg)?;
        crate::services::browser_kernels::migrate_legacy_environment_bindings(&db)
            .await
            .map_err(anyhow::Error::msg)?;

        let local_user_uuid = Self::ensure_local_user(&db).await?;
        let context = Self {
            db,
            workspace_quota: WorkspaceQuotaConfig::default(),
            local_user_uuid,
            session_user_uuid: Arc::new(RwLock::new(None)),
        };
        context.ensure_local_workspace().await?;
        Ok(context)
    }

    /// Build a request-scoped context for the currently authenticated local user.
    /// The database pool and quota configuration stay shared between all users.
    pub fn for_current_user(&self) -> Result<Self, anyhow::Error> {
        let user_uuid = self
            .current_user_uuid()
            .ok_or_else(|| anyhow::anyhow!("No local user is authenticated"))?;
        Ok(self.for_user(user_uuid))
    }

    pub fn for_user(&self, user_uuid: Uuid) -> Self {
        Self {
            db: self.db.clone(),
            workspace_quota: self.workspace_quota.clone(),
            local_user_uuid: user_uuid,
            session_user_uuid: self.session_user_uuid.clone(),
        }
    }

    pub fn current_user_uuid(&self) -> Option<Uuid> {
        *self.session_user_uuid.read().expect("local user session lock poisoned")
    }

    pub fn authenticate_user(&self, user_uuid: Uuid) {
        *self.session_user_uuid.write().expect("local user session lock poisoned") =
            Some(user_uuid);
    }

    pub fn clear_authenticated_user(&self) {
        *self.session_user_uuid.write().expect("local user session lock poisoned") = None;
    }

    pub async fn create_db(config: &DatabaseConfig) -> Result<DbPool, anyhow::Error> {
        database::connect(config).await
    }

    async fn ensure_local_user(db: &DbPool) -> Result<Uuid, anyhow::Error> {
        if let Some(uuid) = sqlx::query_scalar::<_, Uuid>(
            "SELECT uuid FROM users WHERE id = 'LOCAL' AND deleted_at IS NULL LIMIT 1",
        )
        .fetch_optional(db)
        .await?
        {
            sqlx::query(
                "INSERT INTO local_user_auth (user_uuid, avatar) VALUES ($1, '🙂') \
                 ON CONFLICT (user_uuid) DO NOTHING",
            )
            .bind(uuid)
            .execute(db)
            .await?;
            return Ok(uuid);
        }

        let uuid = Uuid::new_v4();
        let mut tx = db.begin().await?;
        sqlx::query("INSERT INTO users (uuid, id) VALUES ($1, 'LOCAL')")
            .bind(uuid)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO user_infos (user_uuid, nickname, email, password) VALUES ($1, $2, $3, '')",
        )
        .bind(uuid)
        .bind("Local User")
        .bind(format!("local-{uuid}@simprint.invalid"))
        .execute(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO local_user_auth (user_uuid, avatar) VALUES ($1, '🙂')")
            .bind(uuid)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(uuid)
    }

    async fn ensure_local_workspace(&self) -> Result<(), anyhow::Error> {
        if crate::models::workspaces::fetch_user_workspaces(&self.db, self.local_user_uuid)
            .await?
            .is_empty()
        {
            crate::services::workspaces::create_workspace_service(
                self,
                self.local_user_uuid,
                &CreateWorkspaceRequest {
                    name: "Local Workspace".to_string(),
                    workspace_type: Some("personal".to_string()),
                },
            )
            .await
            .map_err(anyhow::Error::msg)?;
        }
        Ok(())
    }
}
