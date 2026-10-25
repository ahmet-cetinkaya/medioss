use super::*;
use kernel::UserId;

#[test]
fn accepts_any_user() {
    let query = ExportOwnDataQuery {
        user_id: UserId::new(1),
    };
    assert!(ExportOwnDataQueryValidator.validate(&query).is_ok());
}
