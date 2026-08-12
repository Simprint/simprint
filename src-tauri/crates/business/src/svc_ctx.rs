use crate::{
    database::{self, DbPool},
    entitys::CreateWorkspaceRequest,
    utils::{DatabaseConfig, WorkspaceQuotaConfig},
};
use uuid::Uuid;

/// Shared resources used by handlers and services.
#[derive(Clone)]
pub struct SvcCtx {
    pub db: DbPool,
    pub workspace_quota: WorkspaceQuotaConfig,
    pub local_user_uuid: Uuid,
}

impl SvcCtx {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, anyhow::Error> {
        let db = Self::create_db(config).await?;
        database::migrate(&db).await?;

        let local_user_uuid = Self::ensure_local_user(&db).await?;
        let context = Self {
            db,
            workspace_quota: WorkspaceQuotaConfig::default(),
            local_user_uuid,
        };
        context.ensure_local_workspace().await?;
        Ok(context)
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
