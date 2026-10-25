use super::*;
use kernel::UserId;

#[test]
fn accepts_any_user() {
    let query = ExportMoodRecordsQuery {
        user_id: UserId::new(1),
    };
    assert!(ExportMoodRecordsQueryValidator.validate(&query).is_ok());
}
