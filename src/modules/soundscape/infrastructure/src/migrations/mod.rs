use sea_orm_migration::prelude::*;

mod m20260927_000001_soundscape_create_schema;
mod m20261005_000001_soundscape_sound_icons;
pub use m20260927_000001_soundscape_create_schema::Migration;

pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
pub struct Migrator;
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(Migration),
            Box::new(m20261005_000001_soundscape_sound_icons::Migration),
        ]
    }
    fn migration_table_name() -> DynIden {
        Alias::new("soundscape_migrations").into_iden()
    }
}
