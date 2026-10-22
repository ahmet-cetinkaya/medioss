use crate::ports::practice_sink::PracticeSink;
use chrono::{DateTime, Utc};
use kernel::UserId;

#[derive(Default)]
pub struct FakePracticeSink {
    pub calls: std::sync::Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl PracticeSink for FakePracticeSink {
    async fn record_session(
        &self,
        _u: UserId,
        id: i64,
        _s: DateTime<Utc>,
        _f: DateTime<Utc>,
    ) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("session:{id}"));
        Ok(())
    }
    async fn insert_note(
        &self,
        _u: UserId,
        id: i64,
        _t: &str,
        _c: DateTime<Utc>,
    ) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("note:{id}"));
        Ok(())
    }
    async fn upsert_position(&self, _u: UserId, id: i64, _p: i64) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("position:{id}"));
        Ok(())
    }
    async fn set_completed(&self, _u: UserId, id: i64, _c: bool) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("completed:{id}"));
        Ok(())
    }
}
