mod m20260908_000001_identity_create_schema;
mod m20261002_000001_identity_profile_columns;

use sea_orm::DatabaseConnection;
use sea_orm_migration::{MigrationTrait, MigratorTrait};

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260908_000001_identity_create_schema::Migration),
            Box::new(m20261002_000001_identity_profile_columns::Migration),
        ]
    }
}

pub async fn migrate(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    Migrator::up(db, None).await
}

#[cfg(test)]
#[path = "migrations.tests.rs"]
mod tests;
