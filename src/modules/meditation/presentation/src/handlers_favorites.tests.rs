use axum::http::StatusCode;

use crate::router_tests::*;

async fn send(app: &TestApp, actor: Option<kernel::Actor>, method: &str, path: &str) -> StatusCode {
    json_body(app, actor, method, path, "").await.0
}

async fn put_favorite(app: &TestApp, actor: kernel::Actor, id: i64) -> StatusCode {
    send(
        app,
        Some(actor),
        "PUT",
        &format!("/meditation/{id}/favorite"),
    )
    .await
}

async fn list_favorites(app: &TestApp, actor: kernel::Actor) -> Vec<serde_json::Value> {
    let (status, _, body) = json_body(app, Some(actor), "GET", "/meditation/favorites", "").await;
    assert_eq!(status, StatusCode::OK);
    serde_json::from_slice(&body).unwrap()
}

async fn detail(app: &TestApp, actor: kernel::Actor, id: i64) -> serde_json::Value {
    let (_, _, body) = json_body(app, Some(actor), "GET", &format!("/meditation/{id}"), "").await;
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn favoriting_returns_204_and_marks_the_detail_and_library() {
    let app = app();
    let meditation = seed_meditation(&app).await;

    assert_eq!(
        put_favorite(&app, member_actor(), meditation.id()).await,
        StatusCode::NO_CONTENT
    );

    assert_eq!(
        detail(&app, member_actor(), meditation.id()).await["favorite"],
        true
    );
    let (_, _, body) =
        json_body(&app, Some(member_actor()), "GET", "/meditation/library", "").await;
    let library: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(library[0]["favorite"], true);
}

#[tokio::test]
async fn an_unfavorited_meditation_reports_favorite_false() {
    let app = app();
    let meditation = seed_meditation(&app).await;

    assert_eq!(
        detail(&app, member_actor(), meditation.id()).await["favorite"],
        false
    );
}

#[tokio::test]
async fn favorite_and_unfavorite_are_idempotent() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = format!("/meditation/{}/favorite", meditation.id());

    assert_eq!(
        put_favorite(&app, member_actor(), meditation.id()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        put_favorite(&app, member_actor(), meditation.id()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(list_favorites(&app, member_actor()).await.len(), 1);

    assert_eq!(
        send(&app, Some(member_actor()), "DELETE", &path).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        send(&app, Some(member_actor()), "DELETE", &path).await,
        StatusCode::NO_CONTENT
    );
    assert!(list_favorites(&app, member_actor()).await.is_empty());
}

#[tokio::test]
async fn list_is_newest_favorited_first_and_marks_each_favorite() {
    let app = app();
    let first = seed_meditation(&app).await;
    let second = meditation_domain::Meditation::new(
        2,
        "Second",
        meditation_domain::STORED_CONTENT_TYPE,
        1,
        first.storage_key(),
        60_000,
        chrono::Utc::now(),
    )
    .unwrap();
    app.state.seed_meditation(second.clone());
    put_favorite(&app, member_actor(), first.id()).await;
    // The fake clock is fixed, so equal times fall back to the higher id first.
    put_favorite(&app, member_actor(), second.id()).await;

    let listed = list_favorites(&app, member_actor()).await;

    let ids: Vec<i64> = listed
        .iter()
        .map(|row| row["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![second.id(), first.id()]);
    assert!(listed.iter().all(|row| row["favorite"] == true));
}

#[tokio::test]
async fn another_user_cannot_see_or_inherit_favorites() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    put_favorite(&app, admin_actor(), meditation.id()).await;

    assert!(list_favorites(&app, member_actor()).await.is_empty());
    assert_eq!(
        detail(&app, member_actor(), meditation.id()).await["favorite"],
        false
    );
    assert_eq!(list_favorites(&app, admin_actor()).await.len(), 1);
}

#[tokio::test]
async fn unknown_meditation_is_404_for_both_verbs() {
    let app = app();

    assert_eq!(
        put_favorite(&app, member_actor(), 999).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(
            &app,
            Some(member_actor()),
            "DELETE",
            "/meditation/999/favorite"
        )
        .await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn the_favorites_route_is_not_swallowed_by_the_detail_route() {
    let app = app();

    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/favorites",
        "",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!([])
    );
}

#[tokio::test]
async fn favorites_require_a_session() {
    let app = app();

    assert_eq!(
        send(&app, None, "GET", "/meditation/favorites").await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, None, "PUT", "/meditation/1/favorite").await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, None, "DELETE", "/meditation/1/favorite").await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn mismatched_origin_is_forbidden_for_both_verbs() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    for method in ["PUT", "DELETE"] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            method,
            &format!("/meditation/{}/favorite", meditation.id()),
            vec![("origin", "http://evil.test")],
            axum::body::Body::empty(),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method}");
    }
    assert!(list_favorites(&app, member_actor()).await.is_empty());
}

#[tokio::test]
async fn missing_origin_is_forbidden_for_both_verbs() {
    use tower::ServiceExt;

    let app = app();
    let meditation = seed_meditation(&app).await;
    for method in ["PUT", "DELETE"] {
        let mut request = axum::http::Request::builder()
            .method(method)
            .uri(format!("/meditation/{}/favorite", meditation.id()))
            .body(axum::body::Body::empty())
            .unwrap();
        request.extensions_mut().insert(member_actor());

        let response = app.router.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method}");
    }
    assert!(list_favorites(&app, member_actor()).await.is_empty());
}

#[tokio::test]
async fn each_favorite_names_every_program_it_belongs_to() {
    use meditation_application::ports::{MembershipRepository, ProgramRepository};
    use meditation_application::test_support::{FakeMembershipRepository, FakeProgramRepository};

    let app = app();
    let meditation = seed_meditation(&app).await;
    let programs = FakeProgramRepository(app.state.clone());
    let memberships = FakeMembershipRepository(app.state.clone());
    let now = chrono::Utc::now();
    for (offset_seconds, title) in [(0, "Calm Mornings"), (5, "Deep Sleep")] {
        let program = programs.insert(title.to_owned(), None, now).await.unwrap();
        memberships
            .add(
                program.id,
                meditation.id(),
                now + chrono::Duration::seconds(offset_seconds),
            )
            .await
            .unwrap();
    }
    put_favorite(&app, member_actor(), meditation.id()).await;

    let listed = list_favorites(&app, member_actor()).await;

    assert_eq!(
        listed[0]["programs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|program| program["title"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["Calm Mornings", "Deep Sleep"]
    );
}

#[tokio::test]
async fn a_favorite_without_a_program_lists_an_empty_programs_array() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    put_favorite(&app, member_actor(), meditation.id()).await;

    let listed = list_favorites(&app, member_actor()).await;

    assert_eq!(listed[0]["programs"], serde_json::json!([]));
}
