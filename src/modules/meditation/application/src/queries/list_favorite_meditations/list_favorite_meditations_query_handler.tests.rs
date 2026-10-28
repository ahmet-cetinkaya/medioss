use chrono::Duration;

use super::{ListFavoriteMeditationsQuery, ListFavoriteMeditationsQueryHandler};
use crate::ports::{FavoriteRepository, MembershipRepository, QueryHandler};
use crate::test_support::{user_a, user_b, TestDeps};

fn service(deps: &TestDeps) -> ListFavoriteMeditationsQueryHandler {
    ListFavoriteMeditationsQueryHandler::new(
        deps.meditations.clone(),
        deps.favorites.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
    )
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

    let ids: Vec<i64> = listed.iter().map(|entry| entry.meditation.id()).collect();
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

#[tokio::test]
async fn names_every_program_a_favorite_belongs_to_earliest_membership_first() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let later = deps.seed_program("Later Program").await;
    deps.memberships
        .add(later, meditation.id(), deps.clock.0 + Duration::seconds(5))
        .await
        .unwrap();
    deps.favorites
        .add(user_a(), meditation.id(), deps.clock.0)
        .await
        .unwrap();

    let listed = service(&deps)
        .handle(ListFavoriteMeditationsQuery { user_id: user_a() })
        .await
        .unwrap();

    let titles: Vec<&str> = listed[0]
        .programs
        .iter()
        .map(|program| program.title.as_str())
        .collect();
    assert_eq!(titles, vec!["Holder Program", "Later Program"]);
}

#[tokio::test]
async fn a_favorite_outside_every_program_has_no_program_labels() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    let meditation = deps.seed_meditation_in(program_id, "Seed").await;
    deps.memberships
        .remove(program_id, meditation.id())
        .await
        .unwrap();
    deps.favorites
        .add(user_a(), meditation.id(), deps.clock.0)
        .await
        .unwrap();

    let listed = service(&deps)
        .handle(ListFavoriteMeditationsQuery { user_id: user_a() })
        .await
        .unwrap();

    assert!(listed[0].programs.is_empty());
}
