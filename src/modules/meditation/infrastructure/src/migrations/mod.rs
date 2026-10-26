//! Module-owned migrations with a distinct ledger table so the meditation
//! history versions independently of identity's (ARCHITECTURE.md).

use sea_orm::TransactionTrait;
use sea_orm_migration::prelude::*;

mod m20260926_000001_meditation_create_schema;
mod m20261008_000001_meditation_v1_additions;
mod m20261026_000001_meditation_favorites;

pub use m20260926_000001_meditation_create_schema::Migration;

/// Runs every meditation migration in order against `db`, inside one SQLite
/// transaction: schema changes, backfills, and ledger rows commit together or
/// not at all (sea-orm only does this automatically for Postgres).
pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    let tx = db.begin().await?;
    Migrator::up(&tx, None).await?;
    tx.commit().await
}

/// Ledger isolation: each module's migrator records its own history under a
/// distinct table name (`meditation_migrations`), so no module's migration
/// numbering blocks another's.
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(Migration),
            Box::new(m20261008_000001_meditation_v1_additions::Migration),
            Box::new(m20261026_000001_meditation_favorites::Migration),
        ]
    }

    fn migration_table_name() -> DynIden {
        sea_orm_migration::prelude::Alias::new("meditation_migrations").into_iden()
    }
}

#[cfg(test)]
#[path = "migrations.tests.rs"]
mod tests;
