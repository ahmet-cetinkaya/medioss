use chrono::Duration;

use super::{ListFavoriteMeditationsQuery, ListFavoriteMeditationsQueryHandler};
use crate::ports::{FavoriteRepository, QueryHandler};
use crate::test_support::{user_a, user_b, TestDeps};

fn service(deps: &TestDeps) -> ListFavoriteMeditationsQueryHandler {
    ListFavoriteMeditationsQueryHandler::new(deps.meditations.clone(), deps.favorites.clone())
}

#[tokio::test]
async fn lists_newest_favorited_first_and_only_the_callers() {
    let deps = TestDeps::new();
    let first = deps.seeded_meditation().await;
    let second = deps.seeded_meditation().await;
    let other = deps.seeded_meditation().await;
    let now = deps.clock.0;
    deps.favorites.add(user_a(), first.id(), now).await.unwrap();
    deps.favorites
        .add(user_a(), second.id(), now + Duration::seconds(5))
        .await
        .unwrap();
    deps.favorites.add(user_b(), other.id(), now).await.unwrap();

    let listed = service(&deps)
        .handle(ListFavoriteMeditationsQuery { user_id: user_a() })
        .await
        .unwrap();

    let ids: Vec<i64> = listed.iter().map(|meditation| meditation.id()).collect();
    assert_eq!(ids, vec![second.id(), first.id()]);
}

#[tokio::test]
async fn skips_a_favorite_whose_meditation_no_longer_exists() {
    let deps = TestDeps::new();
    let existing = deps.seeded_meditation().await;
    let now = deps.clock.0;
    deps.favorites.add(user_a(), 999, now).await.unwrap();
    deps.favorites
        .add(user_a(), existing.id(), now - Duration::seconds(1))
        .await
        .unwrap();

    let listed = service(&deps)
        .handle(ListFavoriteMeditationsQuery { user_id: user_a() })
        .await
        .unwrap();

    assert_eq!(listed.len(), 1);
}
