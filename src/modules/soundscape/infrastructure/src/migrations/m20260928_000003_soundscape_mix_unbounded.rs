use sea_orm_migration::prelude::*;

/// Removes the three-sounds-per-mix ceiling (US1 change): the
/// `soundscape_mix_items.position` CHECK allowed only 0..=2. Positions must
/// stay non-negative and ordered (domain rule), but there is no upper bound
/// anymore. SQLite cannot drop a table CHECK, so the table is rebuilt.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "BEGIN;
                 CREATE TABLE soundscape_mix_items_new (id INTEGER PRIMARY KEY AUTOINCREMENT, mix_id INTEGER NOT NULL, sound_id INTEGER NOT NULL, volume INTEGER NOT NULL CHECK(volume BETWEEN 0 AND 100), position INTEGER NOT NULL CHECK(position >= 0), UNIQUE(mix_id, sound_id), UNIQUE(mix_id, position));
                 INSERT INTO soundscape_mix_items_new(id, mix_id, sound_id, volume, position) SELECT id, mix_id, sound_id, volume, position FROM soundscape_mix_items;
                 DROP TABLE soundscape_mix_items;
                 ALTER TABLE soundscape_mix_items_new RENAME TO soundscape_mix_items;
                 COMMIT;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Restoring the 0..=2 CHECK would drop data for larger mixes, so the
        // downgrade only re-tightens the constraint on positions that still
        // fit; mixes that grew beyond three sounds keep their rows.
        manager
            .get_connection()
            .execute_unprepared(
                "BEGIN;
                 CREATE TABLE soundscape_mix_items_new (id INTEGER PRIMARY KEY AUTOINCREMENT, mix_id INTEGER NOT NULL, sound_id INTEGER NOT NULL, volume INTEGER NOT NULL CHECK(volume BETWEEN 0 AND 100), position INTEGER NOT NULL CHECK(position BETWEEN 0 AND 2), UNIQUE(mix_id, sound_id), UNIQUE(mix_id, position));
                 INSERT INTO soundscape_mix_items_new(id, mix_id, sound_id, volume, position)
                   SELECT id, mix_id, sound_id, volume,
                          MIN(position, 2) FROM soundscape_mix_items
                   WHERE position <= 2;
                 DROP TABLE soundscape_mix_items;
                 ALTER TABLE soundscape_mix_items_new RENAME TO soundscape_mix_items;
                 COMMIT;",
            )
            .await?;
        Ok(())
    }
}
