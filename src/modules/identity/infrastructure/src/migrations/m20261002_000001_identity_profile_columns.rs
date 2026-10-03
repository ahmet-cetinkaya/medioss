use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !column_exists(manager, "display_name").await? {
            manager
                .get_connection()
                .execute_unprepared("ALTER TABLE users ADD COLUMN display_name TEXT NULL;")
                .await?;
        }
        if !column_exists(manager, "avatar_ext").await? {
            manager
                .get_connection()
                .execute_unprepared(
                    "ALTER TABLE users ADD COLUMN avatar_ext TEXT NULL CHECK (avatar_ext IN ('webp', 'png'));",
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "downward migrations are not supported".to_owned(),
        ))
    }
}

async fn column_exists(manager: &SchemaManager<'_>, column: &str) -> Result<bool, DbErr> {
    let row = manager
        .get_connection()
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            format!("SELECT 1 FROM pragma_table_info('users') WHERE name = '{column}'"),
        ))
        .await?;
    Ok(row.is_some())
}
