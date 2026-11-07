use sea_orm_migration::prelude::*;

/// Turkish titles for the built-in sounds and categories. `name` keeps the
/// English stored name; `name_tr` is NULL for user content and for built-ins
/// an admin renamed, and the seeder fills it from the manifest.
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
