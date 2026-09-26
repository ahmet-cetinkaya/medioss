use std::{path::Path, time::Duration};

use sea_orm::SqlxSqliteConnector;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};

use crate::bootstrap::BootstrapError;

#[allow(clippy::module_inception)]
mod database;

pub use database::Database;

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const STARTUP_RETRY_DELAY: Duration = Duration::from_millis(10);

pub async fn open(path: &Path) -> Result<Database, BootstrapError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(BUSY_TIMEOUT);
    let pool =
        connect_with_retry(options)
            .await
            .map_err(|source| BootstrapError::DatabaseOpen {
                path: path.to_path_buf(),
                source,
            })?;
    let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());

    Ok(Database { connection, pool })
}

async fn connect_with_retry(options: SqliteConnectOptions) -> Result<SqlitePool, sqlx::Error> {
    let deadline = tokio::time::Instant::now() + BUSY_TIMEOUT;

    loop {
        match SqlitePool::connect_with(options.clone()).await {
            Err(error) if is_busy(&error) && tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(STARTUP_RETRY_DELAY).await;
            }
            result => return result,
        }
    }
}

fn is_busy(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| code.parse::<i32>().is_ok_and(|code| code & 0xff == 5))
}

#[cfg(test)]
#[path = "database.tests.rs"]
mod tests;
