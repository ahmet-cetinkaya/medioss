use crate::bootstrap::{database::Database, report::fail};

/// Runs every module's migrator in a fixed order; any failure aborts startup.
pub async fn migrate_all(database: &Database) {
    if let Err(err) = identity_infrastructure::migrate(&database.connection).await {
        fail(format!("identity migration failed: {err}"));
    }
    if let Err(err) = meditation_infrastructure::migrations::migrate(&database.connection).await {
        fail(format!("meditation migration failed: {err}"));
    }
    if let Err(err) = soundscape_infrastructure::migrations::migrate(&database.connection).await {
        fail(format!("soundscape migration failed: {err}"));
    }
    if let Err(err) = mood_tracker_infrastructure::migrations::migrate(&database.connection).await {
        fail(format!("mood tracker migration failed: {err}"));
    }
}

#[cfg(test)]
#[path = "migrations.tests.rs"]
mod tests;
