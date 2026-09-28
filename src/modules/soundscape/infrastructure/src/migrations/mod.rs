use sea_orm_migration::prelude::*;

mod m20260927_000001_soundscape_create_schema;
mod m20260928_000002_soundscape_add_builtin_id;
mod m20260928_000003_soundscape_mix_unbounded;
mod m20260928_000004_soundscape_builtin_tombstones;
pub use m20260927_000001_soundscape_create_schema::Migration;
pub use m20260928_000002_soundscape_add_builtin_id::Migration as AddBuiltinIdMigration;
pub use m20260928_000003_soundscape_mix_unbounded::Migration as MixUnboundedMigration;
pub use m20260928_000004_soundscape_builtin_tombstones::Migration as BuiltinTombstonesMigration;

pub async fn migrate(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
pub struct Migrator;
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(Migration),
            Box::new(AddBuiltinIdMigration),
            Box::new(MixUnboundedMigration),
            Box::new(BuiltinTombstonesMigration),
        ]
    }
    fn migration_table_name() -> DynIden {
        Alias::new("soundscape_migrations").into_iden()
    }
}
