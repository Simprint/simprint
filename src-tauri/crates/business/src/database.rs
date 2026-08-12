use std::str::FromStr;

use sqlx::{
    Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use crate::utils::DatabaseConfig;

/// Database engine used by the embedded business layer.
pub type Db = Sqlite;
pub type Pool<T = Db> = sqlx::Pool<T>;
pub type DbPool = Pool<Db>;

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
}
