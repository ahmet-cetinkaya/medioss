use kernel::UserId;

use super::super::get_playback_state_query::GetPlaybackStateQuery;
use super::GetPlaybackStateQueryValidator;

#[test]
fn accepts_the_query() {
    assert!(GetPlaybackStateQueryValidator
        .validate(&GetPlaybackStateQuery {
            user_id: UserId::new(1),
            meditation_id: 1
        })
        .is_ok());
}
