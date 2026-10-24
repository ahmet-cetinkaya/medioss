use std::sync::Arc;

use super::{IsSetupRequired, IsSetupRequiredHandler, SetupStatusService};
use crate::test_support::{block_on, FakeRepository};

#[test]
fn setup_is_required_only_for_an_empty_repository() {
    let empty = Arc::new(FakeRepository::new());
    let initialized = Arc::new(FakeRepository::with_permanent_admin());

    assert!(
        block_on(SetupStatusService::new(empty).handle(IsSetupRequired)).unwrap(),
        "an empty instance requires setup"
    );
    assert!(
        !block_on(SetupStatusService::new(initialized).handle(IsSetupRequired)).unwrap(),
        "an initialized instance does not require setup"
    );
}
