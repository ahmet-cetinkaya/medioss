use sea_orm_migration::prelude::*;

/// Creates the whole soundscape schema in its final, pre-release shape:
/// unbounded mix positions, built-in seeding markers, and built-in
/// tombstones. One migration because the module has not shipped yet.
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for sql in [
   "CREATE TABLE IF NOT EXISTS soundscape_categories (id INTEGER PRIMARY KEY AUTOINCREMENT, slug TEXT NOT NULL UNIQUE, name TEXT NOT NULL COLLATE NOCASE UNIQUE);",
   "CREATE TABLE IF NOT EXISTS soundscape_sounds (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, category_id INTEGER NOT NULL, content_type TEXT NOT NULL, size_bytes INTEGER NOT NULL, storage_key TEXT NOT NULL UNIQUE, duration_ms INTEGER NOT NULL, created_at TEXT NOT NULL, builtin_id TEXT);",
   "CREATE UNIQUE INDEX IF NOT EXISTS soundscape_sounds_builtin_id_idx ON soundscape_sounds(builtin_id) WHERE builtin_id IS NOT NULL;",
   "CREATE TABLE IF NOT EXISTS soundscape_mixes (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER NOT NULL, name TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);",
   "CREATE TABLE IF NOT EXISTS soundscape_mix_items (id INTEGER PRIMARY KEY AUTOINCREMENT, mix_id INTEGER NOT NULL, sound_id INTEGER NOT NULL, volume INTEGER NOT NULL CHECK(volume BETWEEN 0 AND 100), position INTEGER NOT NULL CHECK(position >= 0), UNIQUE(mix_id, sound_id), UNIQUE(mix_id, position));",
   "CREATE TABLE IF NOT EXISTS soundscape_builtin_tombstones (builtin_id TEXT PRIMARY KEY, deleted_at TEXT NOT NULL);",
   "CREATE INDEX IF NOT EXISTS soundscape_sounds_category_idx ON soundscape_sounds(category_id, name, id);",
   "CREATE INDEX IF NOT EXISTS soundscape_mixes_user_idx ON soundscape_mixes(user_id, updated_at DESC, id DESC);",
   "INSERT OR IGNORE INTO soundscape_categories (id, slug, name) VALUES (1, 'nature', 'Nature'), (2, 'mechanical', 'Mechanical'), (3, 'ambient', 'Ambient');"
  ] { manager.get_connection().execute_unprepared(sql).await?; }
        Ok(())
    }
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
