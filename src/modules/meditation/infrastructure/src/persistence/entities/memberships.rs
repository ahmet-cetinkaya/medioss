//! SeaORM entity for `meditation_program_memberships`.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meditation_program_memberships")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub program_id: i64,
    pub meditation_id: i64,
    pub added_at: String,
    pub position: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
