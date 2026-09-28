use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE soundscape_sounds ADD COLUMN builtin_id TEXT;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                // SQLite cannot ADD a UNIQUE column in place; a unique index gives the
                // same duplicate rejection without a table rebuild.
                "CREATE UNIQUE INDEX IF NOT EXISTS soundscape_sounds_builtin_id_idx ON soundscape_sounds(builtin_id);",
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS soundscape_sounds_builtin_id_idx;
                 CREATE TABLE IF NOT EXISTS soundscape_sounds_mig (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, category_id INTEGER NOT NULL, content_type TEXT NOT NULL, size_bytes INTEGER NOT NULL, storage_key TEXT NOT NULL UNIQUE, duration_ms INTEGER NOT NULL, created_at TEXT NOT NULL);
                 INSERT OR IGNORE INTO soundscape_sounds_mig SELECT id, name, category_id, content_type, size_bytes, storage_key, duration_ms, created_at FROM soundscape_sounds;
                 DROP TABLE soundscape_sounds;
                 ALTER TABLE soundscape_sounds_mig RENAME TO soundscape_sounds;
                 CREATE INDEX IF NOT EXISTS soundscape_sounds_category_idx ON soundscape_sounds(category_id, name, id);",
            )
            .await?;
        Ok(())
    }
}
