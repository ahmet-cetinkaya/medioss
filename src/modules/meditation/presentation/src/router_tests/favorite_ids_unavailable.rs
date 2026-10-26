//! The `favorite` flag is decoration: when the caller's favorites cannot be
//! read, every surface that carries it still answers, with `favorite: false`.

use std::collections::HashSet;
use std::sync::Arc;

use axum::http::StatusCode;
use meditation_application::ports::QueryHandler;
use meditation_application::queries::list_favorite_ids::ListFavoriteIdsQuery;
use meditation_application::test_support::{FakeBlobStore, RepoState};
use meditation_application::ApplicationError;

use super::helpers::*;
use crate::meditation_router;

struct UnreadableFavorites;

#[async_trait::async_trait]
impl QueryHandler<ListFavoriteIdsQuery> for UnreadableFavorites {
    type Output = HashSet<i64>;

    async fn handle(&self, _query: ListFavoriteIdsQuery) -> Result<Self::Output, ApplicationError> {
        Err(ApplicationError::Repository("favorites store down".into()))
    }
}

fn app_with_unreadable_favorites() -> TestApp {
    let state = RepoState::new();
    let store = FakeBlobStore::new();
    let mut deps = test_dependencies(state.clone(), &store);
    deps.list_favorite_ids = Arc::new(UnreadableFavorites);
    TestApp {
        router: meditation_router(TEST_MAX_UPLOAD_BYTES, TEST_MAX_COVER_BYTES)
            .with_state(Arc::new(deps)),
        state,
        store,
    }
}

async fn get_json(app: &TestApp, path: &str) -> serde_json::Value {
    let (status, _, body) = json_body(app, Some(member_actor()), "GET", path, "").await;
    assert_eq!(status, StatusCode::OK, "{path}");
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn library_detail_daily_suggestion_and_program_members_still_answer() {
    let app = app_with_unreadable_favorites();
    seed_search_library(&app).await;

    let library = get_json(&app, "/meditation/library").await;
    let entries = library.as_array().unwrap();
    assert!(!entries.is_empty());
    assert!(entries.iter().all(|entry| entry["favorite"] == false));

    assert_eq!(get_json(&app, "/meditation/5").await["favorite"], false);

    let suggestion = get_json(&app, "/meditation/daily-suggestion").await;
    assert_eq!(suggestion["favorite"], false);

    let members = get_json(&app, "/meditation/programs/2/members").await;
    assert!(!members.as_array().unwrap().is_empty());
    assert!(members
        .as_array()
        .unwrap()
        .iter()
        .all(|member| member["favorite"] == false));
}
