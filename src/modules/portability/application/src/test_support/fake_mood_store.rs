use crate::models::mood_record::MoodRecord;
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use kernel::UserId;

#[derive(Default)]
pub struct FakeMoodStore {
    pub entries: std::sync::Mutex<Vec<MoodRecord>>,
}

#[async_trait::async_trait]
impl OwnMoodRecords for FakeMoodStore {
    async fn list_for_user(&self, _user: UserId) -> Result<Vec<MoodRecord>, String> {
        Ok(self.entries.lock().unwrap().clone())
    }
}

#[async_trait::async_trait]
impl MoodRecordSink for FakeMoodStore {
    async fn insert(&self, _user: UserId, record: MoodRecord) -> Result<(), String> {
        self.entries.lock().unwrap().push(record);
        Ok(())
    }
}

pub struct MoodSinkFailingOnSecondInsert {
    pub inserted: std::sync::Mutex<Vec<MoodRecord>>,
}

#[async_trait::async_trait]
impl MoodRecordSink for MoodSinkFailingOnSecondInsert {
    async fn insert(&self, _user: UserId, record: MoodRecord) -> Result<(), String> {
        let mut inserted = self.inserted.lock().unwrap();
        if inserted.len() == 1 {
            return Err("disk full".into());
        }
        inserted.push(record);
        Ok(())
    }
}
