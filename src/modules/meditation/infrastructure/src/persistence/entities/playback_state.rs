//! SeaORM entity for `meditation_playback_state` (composite primary key:
//! user_id + meditation_id, both marked primary_key per SeaORM convention).
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "meditation_playback_state")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub meditation_id: i64,
    pub position_ms: i64,
    pub completed: bool,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
