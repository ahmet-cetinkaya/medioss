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

pub async fn open(path: &Path) -> Result<Database, BootstrapError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    let pool =
        SqlitePool::connect_with(options)
            .await
            .map_err(|source| BootstrapError::DatabaseOpen {
                path: path.to_path_buf(),
                source,
            })?;
    let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());

    Ok(Database { connection, pool })
}

#[cfg(test)]
#[path = "database.tests.rs"]
mod tests;
