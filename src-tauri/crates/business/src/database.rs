use std::str::FromStr;

use sqlx::{
    Sqlite,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use crate::utils::DatabaseConfig;

/// Database engine used by the embedded business layer.
pub type Db = Sqlite;
pub type Pool<T = Db> = sqlx::Pool<T>;
pub type DbPool = Pool<Db>;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Build a numbered placeholder list for a variable-length SQLite `IN` clause.
pub fn placeholders(start: usize, len: usize) -> String {
    (start..start + len)
        .map(|index| format!("${index}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<DbPool> {
    let options = SqliteConnectOptions::from_str(&config.url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout))
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_with(options)
        .await?;

    Ok(pool)
}

/// Bring a newly-created or existing embedded database up to the current schema.
pub async fn migrate(pool: &DbPool) -> anyhow::Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_an_embedded_sqlite_database() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
            max_lifetime: 30,
            acquire_timeout: 30,
            idle_timeout: 30,
        };

        let pool = connect(&config).await.expect("SQLite should open");
        sqlx::query("CREATE TABLE health_check (value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("schema should be writable");
        sqlx::query("INSERT INTO health_check (value) VALUES ($1)")
            .bind("ok")
            .execute(&pool)
            .await
            .expect("data should be writable");

        let value: String = sqlx::query_scalar("SELECT value FROM health_check")
            .fetch_one(&pool)
            .await
            .expect("data should be readable");
        assert_eq!(value, "ok");
        assert_eq!(placeholders(3, 3), "$3, $4, $5");
    }

    #[tokio::test]
    async fn applies_all_migrations_to_a_fresh_database() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
            max_lifetime: 30,
            acquire_timeout: 30,
            idle_timeout: 30,
        };

        let pool = connect(&config).await.expect("SQLite should open");
        migrate(&pool).await.expect("all embedded migrations should apply");

        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_one(&pool)
        .await
        .expect("schema metadata should be readable");
        assert!(table_count > 20, "business tables should be created");
    }

    #[tokio::test]
    async fn core_models_round_trip_on_sqlite() {
        use crate::entitys::{CreateTeamRequest, CreateWorkspaceRequest};
        use crate::models::{environments, proxies, teams, workspaces};

        let mut config = DatabaseConfig::embedded("sqlite::memory:");
        config.max_connections = 1;
        config.min_connections = 1;
        let context = crate::svc_ctx::SvcCtx::new(&config)
            .await
            .expect("business context should initialize");
        let pool = &context.db;
        let user_uuid = uuid::Uuid::new_v4();

        sqlx::query("INSERT INTO users (uuid, id) VALUES ($1, $2)")
            .bind(user_uuid)
            .bind("LOCAL_USER")
            .execute(pool)
            .await
            .expect("local user should be inserted");
        sqlx::query(
            "INSERT INTO user_infos (user_uuid, nickname, email, password) VALUES ($1, $2, $3, $4)",
        )
        .bind(user_uuid)
        .bind("Local User")
        .bind("local@simprint.invalid")
        .bind("")
        .execute(pool)
        .await
        .expect("local user profile should be inserted");

        let workspace_uuid = workspaces::insert_workspace(
            pool,
            user_uuid,
            &CreateWorkspaceRequest {
                name: "Local Workspace".to_string(),
                workspace_type: Some("personal".to_string()),
            },
        )
        .await
        .expect("workspace should be created");
        let workspace = workspaces::fetch_workspace_by_uuid(pool, workspace_uuid)
            .await
            .expect("workspace query should succeed")
            .expect("workspace should exist");
        assert_eq!(workspace.name, "Local Workspace");

        let team_uuid = teams::insert_team(
            pool,
            user_uuid,
            &CreateTeamRequest {
                workspace_uuid,
                name: "Local Team".to_string(),
                description: None,
            },
        )
        .await
        .expect("team should be created");
        assert!(teams::fetch_team_by_uuid(pool, team_uuid).await.unwrap().is_some());

        let group_uuid =
            environments::insert_group(pool, workspace_uuid, team_uuid, "Default", None, user_uuid)
                .await
                .expect("group should be created");
        let tag_uuid =
            environments::insert_tag(pool, user_uuid, Some(team_uuid), "QA", Some("blue"))
                .await
                .expect("tag should be created");
        assert!(environments::fetch_group_by_uuid(pool, group_uuid).await.unwrap().is_some());
        assert!(environments::fetch_tag_by_uuid(pool, tag_uuid).await.unwrap().is_some());

        let proxy_uuid = proxies::insert_proxy(
            pool,
            workspace_uuid,
            user_uuid,
            "Local Proxy",
            "127.0.0.1",
            8080,
            "http",
            None,
            Some("secret"),
            None,
            None,
        )
        .await
        .expect("proxy should be created");
        let proxy = proxies::fetch_proxy_by_uuid(pool, proxy_uuid)
            .await
            .expect("proxy query should succeed")
            .expect("proxy should exist");
        assert_eq!(proxy.password.as_deref(), Some("secret"));

        let environment_uuid = environments::insert_environment(
            pool,
            workspace_uuid,
            user_uuid,
            team_uuid,
            "Browser A",
            None,
            Some(group_uuid),
            Some(proxy_uuid),
            Some("Windows"),
            Some("Chromium"),
        )
        .await
        .expect("environment should be created");
        let environment =
            environments::fetch_environment_by_uuid(pool, workspace_uuid, environment_uuid)
                .await
                .expect("environment query should succeed")
                .expect("environment should exist");
        assert_eq!(environment.name, "Browser A");
        assert_eq!(environment.proxy_uuid, Some(proxy_uuid));

        workspaces::update_workspace(pool, workspace_uuid, Some("Renamed Workspace"))
            .await
            .expect("workspace should update");
        assert_eq!(
            workspaces::fetch_workspace_by_uuid(pool, workspace_uuid)
                .await
                .unwrap()
                .unwrap()
                .name,
            "Renamed Workspace"
        );
    }
}
