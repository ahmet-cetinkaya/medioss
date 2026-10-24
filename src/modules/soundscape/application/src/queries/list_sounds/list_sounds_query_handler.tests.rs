use super::*;
use crate::test_support::{sample_sound, InMemorySounds};

#[tokio::test]
async fn lists_every_sound() {
    let sounds = InMemorySounds::with(vec![sample_sound(1), sample_sound(2)]);
    let listed = ListSoundsQueryHandler::new(Arc::new(sounds))
        .handle(ListSoundsQuery)
        .await
        .unwrap();
    let ids: Vec<_> = listed.iter().map(|sound| sound.id).collect();
    assert_eq!(ids, vec![1, 2]);
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = ListSoundsQueryHandler::new(Arc::new(InMemorySounds::failing("boom")))
        .handle(ListSoundsQuery)
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
