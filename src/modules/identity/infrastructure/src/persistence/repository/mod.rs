mod login_throttle_repository;
mod timestamps;
mod user_repository;
mod user_row;

use sea_orm::DatabaseConnection;

pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
