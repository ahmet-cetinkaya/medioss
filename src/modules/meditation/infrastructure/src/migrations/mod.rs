//! Module-owned migrations with a distinct ledger table so the meditation
//! history versions independently of identity's (ARCHITECTURE.md).

use sea_orm_migration::prelude::*;

mod m20260926_000001_meditation_create_schema;

pub use m20260926_000001_meditation_create_schema::Migration;

/// Runs every meditation migration in order against `db`.
pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}

/// Ledger isolation: each module's migrator records its own history under a
/// distinct table name (`meditation_migrations`), so no module's migration
/// numbering blocks another's.
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(Migration)]
    }

    fn migration_table_name() -> DynIden {
        sea_orm_migration::prelude::Alias::new("meditation_migrations").into_iden()
    }
}

#[cfg(test)]
#[path = "migrations.tests.rs"]
mod tests;
