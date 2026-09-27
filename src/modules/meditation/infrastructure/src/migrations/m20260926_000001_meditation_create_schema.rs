use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // One SQLite file, module-owned tables (ARCHITECTURE.md "Runtime Data
        // and Media Security"): no cross-module foreign keys, user_id is a
        // plain INTEGER column validated by the owning module, never by the
        // database.
        for table in [
            // The shared library rows. The stored form is always Ogg Opus,
            // so content_type is the constant the domain pins.
            "CREATE TABLE IF NOT EXISTS meditation_meditations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                content_type TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                storage_key TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS meditation_programs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                folder TEXT NOT NULL UNIQUE,
                description TEXT NULL,
                cover_storage_key TEXT NULL,
                category_id INTEGER NULL,
                created_at TEXT NOT NULL
            );",
            // Instance-wide unique: categories are admin-owned shared
            // structure, not per-user (spec FR-030).
            "CREATE TABLE IF NOT EXISTS meditation_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS meditation_program_memberships (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                program_id INTEGER NOT NULL,
                meditation_id INTEGER NOT NULL,
                added_at TEXT NOT NULL,
                UNIQUE (program_id, meditation_id)
            );",
            "CREATE TABLE IF NOT EXISTS meditation_playback_state (
                user_id INTEGER NOT NULL,
                meditation_id INTEGER NOT NULL,
                position_ms INTEGER NOT NULL DEFAULT 0,
                completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1)),
                updated_at TEXT NOT NULL,
                PRIMARY KEY (user_id, meditation_id)
            );",
            // The UNIQUE triple is the completion-report idempotency key
            // (spec FR-021, plan D8).
            "CREATE TABLE IF NOT EXISTS meditation_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                meditation_id INTEGER NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT NOT NULL,
                UNIQUE (user_id, meditation_id, started_at)
            );",
            "CREATE TABLE IF NOT EXISTS meditation_notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                meditation_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        ] {
            manager.get_connection().execute_unprepared(table).await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Deliberately irreversible, mirroring identity's migration: dropping
        // module data must never be one accidental `down` away.
        Err(DbErr::Custom(
            "meditation migrations are irreversible; restore from a data-dir backup".to_owned(),
        ))
    }
}
