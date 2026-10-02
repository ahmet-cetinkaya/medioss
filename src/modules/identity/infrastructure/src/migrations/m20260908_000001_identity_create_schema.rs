use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    email TEXT NOT NULL COLLATE NOCASE,
                    password_hash TEXT NOT NULL,
                    role TEXT NOT NULL,
                    must_change_password INTEGER NOT NULL DEFAULT 0 CHECK (must_change_password IN (0, 1)),
                    auth_version INTEGER NOT NULL DEFAULT 0,
                    password_expires_at TEXT NULL,
                    disabled INTEGER NOT NULL DEFAULT 0 CHECK (disabled IN (0, 1)),
                    created_at TEXT NOT NULL,
                    UNIQUE (email)
                );",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS instance_state (
                    singleton INTEGER PRIMARY KEY CHECK (singleton = 1)
                );",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO instance_state (singleton) VALUES (1) ON CONFLICT DO NOTHING;",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS login_attempts (
                    normalized_identifier TEXT PRIMARY KEY,
                    failed_attempt_count INTEGER NOT NULL DEFAULT 0,
                    last_failed_attempt_at TEXT NULL,
                    blocked_until TEXT NULL
                );",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Deliberately irreversible: reverting would destroy every account,
        // password hash, and the setup-claim state. Restore from a backup
        // instead (see docs/DEPLOYMENT.md).
        Err(DbErr::Custom(String::from(
            "identity migration is irreversible; restore from backup instead",
        )))
    }
}
