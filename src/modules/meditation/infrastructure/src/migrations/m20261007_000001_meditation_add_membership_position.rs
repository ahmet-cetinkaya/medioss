use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE meditation_program_memberships \
             ADD COLUMN position INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        // Backfill: current added_at order (row id breaks ties) per program.
        db.execute_unprepared(
            "UPDATE meditation_program_memberships SET position = (\
               SELECT COUNT(*) FROM meditation_program_memberships AS o \
               WHERE o.program_id = meditation_program_memberships.program_id \
                 AND (o.added_at < meditation_program_memberships.added_at \
                   OR (o.added_at = meditation_program_memberships.added_at \
                       AND o.id < meditation_program_memberships.id)))",
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
