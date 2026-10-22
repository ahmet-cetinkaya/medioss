use crate::models::practice_snapshot::PracticeSnapshot;
use crate::ports::own_practice_records::OwnPracticeRecords;
use kernel::UserId;

pub struct FakePracticeSource;

#[async_trait::async_trait]
impl OwnPracticeRecords for FakePracticeSource {
    async fn list_for_user(&self, _user: UserId) -> Result<PracticeSnapshot, String> {
        Ok(PracticeSnapshot::default())
    }
}
