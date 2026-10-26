use sea_orm_migration::prelude::*;

/// Per-user favorite meditations. Like every meditation table there is no
/// foreign key: `user_id` belongs to identity, and deleting a meditation or a
/// user purges these rows through the application layer.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS meditation_favorites (
                    user_id INTEGER NOT NULL,
                    meditation_id INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (user_id, meditation_id)
                );",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "meditation migrations are irreversible; restore from a data-dir backup".to_owned(),
        ))
    }
}
