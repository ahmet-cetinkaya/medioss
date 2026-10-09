use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Optional admin-written text; NULL means "no description".
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE meditation_meditations ADD COLUMN description TEXT NULL",
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
