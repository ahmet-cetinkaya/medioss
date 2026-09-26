use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "login_attempts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub normalized_identifier: String,
    pub failed_attempt_count: i64,
    pub last_failed_attempt_at: Option<String>,
    pub blocked_until: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
