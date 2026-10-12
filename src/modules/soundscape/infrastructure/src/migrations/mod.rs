use sea_orm::TransactionTrait;
use sea_orm_migration::prelude::*;

mod m20260927_000001_soundscape_create_schema;
mod m20261008_000001_soundscape_v1_additions;
pub use m20260927_000001_soundscape_create_schema::Migration;

/// Runs pending migrations inside one SQLite transaction, so schema changes,
/// backfills, and the ledger row commit together or not at all. sea-orm only
/// does this automatically for Postgres.
pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    let tx = db.begin().await?;
    Migrator::up(&tx, None).await?;
    tx.commit().await
}
pub struct Migrator;
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(Migration),
            Box::new(m20261008_000001_soundscape_v1_additions::Migration),
        ]
    }
    fn migration_table_name() -> DynIden {
        Alias::new("soundscape_migrations").into_iden()
    }
}

#[cfg(test)]
#[path = "migrations.tests.rs"]
mod tests;
