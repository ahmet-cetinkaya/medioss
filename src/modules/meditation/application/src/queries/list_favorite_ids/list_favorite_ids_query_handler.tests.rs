use super::{ListFavoriteIdsQuery, ListFavoriteIdsQueryHandler};
use crate::ports::{FavoriteRepository, QueryHandler};
use crate::test_support::{user_a, user_b, TestDeps};

#[tokio::test]
async fn returns_only_the_callers_favorite_ids() {
    let deps = TestDeps::new();
    let now = deps.clock.0;
    deps.favorites.add(user_a(), 1, now).await.unwrap();
    deps.favorites.add(user_a(), 2, now).await.unwrap();
    deps.favorites.add(user_b(), 3, now).await.unwrap();

    let ids = ListFavoriteIdsQueryHandler::new(deps.favorites.clone())
        .handle(ListFavoriteIdsQuery { user_id: user_a() })
        .await
        .unwrap();

    assert_eq!(ids, [1, 2].into_iter().collect());
}
