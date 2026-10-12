use crate::icon_defaults::default_icon_key;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

/// One migration for every pre-release v1 addition (still pre-1.0.0): the
/// bundled-glyph key and the uploaded-icon location on sounds, plus the
/// icon-key backfill for existing rows from the shared default mapping.
/// `migrate` wraps it in a transaction so a crash mid-run cannot leave schema
/// without its ledger row.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for sql in [
            "ALTER TABLE soundscape_sounds ADD COLUMN icon_key TEXT NULL;",
            "ALTER TABLE soundscape_sounds ADD COLUMN icon_storage_key TEXT NULL;",
            "ALTER TABLE soundscape_sounds ADD COLUMN icon_content_type TEXT NULL;",
        ] {
            db.execute_unprepared(sql).await?;
        }
        let rows = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT s.id AS id, s.name AS name, c.slug AS slug FROM soundscape_sounds s JOIN soundscape_categories c ON c.id = s.category_id".to_owned(),
            ))
            .await?;
        for row in rows {
            let id: i64 = row.try_get("", "id")?;
            let name: String = row.try_get("", "name")?;
            let slug: String = row.try_get("", "slug")?;
            if let Some(key) = default_icon_key(&slug, &name) {
                db.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "UPDATE soundscape_sounds SET icon_key = ? WHERE id = ?",
                    [key.into(), id.into()],
                ))
                .await?;
            }
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "soundscape migrations are irreversible; restore from a data-dir backup".to_owned(),
        ))
    }
}
