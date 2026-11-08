use sea_orm_migration::prelude::*;

/// Adds `name_tr` columns that nothing reads or writes any more. The titles
/// moved to the web UI, but this migration shipped, so databases that applied
/// it list it in the ledger and the migrator refuses to start without its file.
/// It stays so those databases keep opening; the columns stay empty.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for sql in [
            "ALTER TABLE soundscape_sounds ADD COLUMN name_tr TEXT NULL;",
            "ALTER TABLE soundscape_categories ADD COLUMN name_tr TEXT NULL;",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for sql in [
            "ALTER TABLE soundscape_categories DROP COLUMN name_tr;",
            "ALTER TABLE soundscape_sounds DROP COLUMN name_tr;",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}
