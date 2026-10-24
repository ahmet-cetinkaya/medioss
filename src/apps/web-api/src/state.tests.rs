use std::sync::Arc;
use std::time::Duration;

use super::{LoginAdmission, TokioSemaphoreAdmission};

#[test]
fn second_acquire_fails_while_a_permit_is_held_and_succeeds_after_drop() {
    let admission: Arc<dyn LoginAdmission> = Arc::new(TokioSemaphoreAdmission::new(1));

    let permit = admission
        .try_acquire()
        .expect("a fresh semaphore with bound 1 has one slot");

    let retry_after = match admission.try_acquire() {
        Err(retry_after) => retry_after,
        Ok(_unexpected_permit) => {
            panic!("the held permit must keep the only slot taken")
        }
    };
    assert_eq!(retry_after, Duration::from_millis(250));

    drop(permit);
    admission
        .try_acquire()
        .expect("dropping the permit must release the slot");
}
