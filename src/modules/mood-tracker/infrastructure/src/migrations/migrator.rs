use sea_orm_migration::prelude::*;

use super::Migration;

pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(Migration)]
    }

    fn migration_table_name() -> DynIden {
        Alias::new("mood_tracker_migrations").into_iden()
    }
}
