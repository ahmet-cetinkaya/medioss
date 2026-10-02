use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub role: String,
    #[sea_orm(column_type = "Boolean")]
    pub must_change_password: bool,
    pub auth_version: i64,
    pub password_expires_at: Option<String>,
    #[sea_orm(column_type = "Boolean")]
    pub disabled: bool,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
