use sea_orm::DatabaseConnection;
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct Database {
    pub connection: DatabaseConnection,
    pub pool: SqlitePool,
}
