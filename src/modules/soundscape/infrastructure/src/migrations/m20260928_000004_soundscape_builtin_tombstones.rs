use sea_orm_migration::prelude::*;

/// Remembers built-in sounds an admin removed so the startup seeder does not
/// resurrect them on the next boot (FR-029: built-ins are ordinary library
/// rows an admin may remove — removal must stick).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS soundscape_builtin_tombstones (builtin_id TEXT PRIMARY KEY, deleted_at TEXT NOT NULL);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS soundscape_builtin_tombstones;")
            .await?;
        Ok(())
    }
}
