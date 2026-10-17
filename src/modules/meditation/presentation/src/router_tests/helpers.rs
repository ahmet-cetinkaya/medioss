use std::sync::Arc;

use chrono::Utc;

use axum::{
    body::Body,
    http::{HeaderName, Request, StatusCode},
    Router,
};
use kernel::{Actor, Role, UserId};
use tower::ServiceExt;

use meditation_application::test_support::{
    FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
    FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository, FakePlaybackRepository,
    FakeProgramRepository, FakeSessionRepository, FakeValidator, FixedClock, RepoState,
};

use crate::{meditation_router, MeditationDependencies};

pub(super) fn admin_actor() -> Actor {
    Actor::new(UserId::new(1), Role::Admin)
}

pub(super) fn member_actor() -> Actor {
    Actor::new(UserId::new(2), Role::Member)
}

pub(super) struct TestApp {
    pub(super) router: Router,
    pub(super) state: RepoState,
    pub(super) store: FakeBlobStore,
}

pub(super) fn app() -> TestApp {
    let state = RepoState::new();
    let store = FakeBlobStore::new();
    let deps = MeditationDependencies {
        meditations: Arc::new(FakeMeditationRepository(state.clone())),
        programs: Arc::new(FakeProgramRepository(state.clone())),
        categories: Arc::new(FakeCategoryRepository(state.clone())),
        memberships: Arc::new(FakeMembershipRepository(state.clone())),
        notes: Arc::new(FakeNoteRepository(state.clone())),
        playback: Arc::new(FakePlaybackRepository(state.clone())),
        sessions: Arc::new(FakeSessionRepository(state.clone())),
        validator: Arc::new(FakeValidator),
        cover_validator: Arc::new(FakeCoverValidator),
        converter: Arc::new(FakeConverter { fail: false }),
        store: Arc::new(store.clone()),
        clock: Arc::new(FixedClock(chrono::Utc::now())),
        medioss_origin: Arc::new("http://localhost".to_owned()),
    };
    TestApp {
        router: meditation_router().with_state(Arc::new(deps)),
        state,
        store,
    }
}

pub(super) async fn call(
    app: &TestApp,
    actor: Option<Actor>,
    method: &str,
    path: &str,
    headers: Vec<(&str, &str)>,
    body: Body,
) -> (StatusCode, Vec<(HeaderName, String)>, Vec<u8>) {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(name, value);
    }
    request = request.header("origin", "http://localhost");
    let mut request = request.body(body).unwrap();
    if let Some(actor) = actor {
        request.extensions_mut().insert(actor);
    }
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| (name.clone(), value.to_str().unwrap_or("").to_owned()))
        .collect();
    let body = axum::body::to_bytes(response.into_body(), 128 * 1024 * 1024)
        .await
        .unwrap()
        .to_vec();
    (status, headers, body)
}

pub(super) async fn json_body(
    app: &TestApp,
    actor: Option<Actor>,
    method: &str,
    path: &str,
    json: &str,
) -> (StatusCode, Vec<(HeaderName, String)>, Vec<u8>) {
    call(
        app,
        actor,
        method,
        path,
        vec![("content-type", "application/json")],
        Body::from(json.to_owned()),
    )
    .await
}

pub(super) async fn seed_search_library(app: &TestApp) {
    use meditation_application::ports::{
        CategoryRepository, MeditationRepository, MembershipRepository, ProgramRepository,
    };
    let meditations = FakeMeditationRepository(app.state.clone());
    let programs = FakeProgramRepository(app.state.clone());
    let categories = FakeCategoryRepository(app.state.clone());
    let memberships = FakeMembershipRepository(app.state.clone());
    let now = Utc::now();
    for i in 0..120 {
        let name = match i % 6 {
            0 => "Morning Breathing",
            1 => "Café Quietude",
            2 => "Evening Calm",
            3 => "Dreamscape",
            4 => "Breathing Exercise",
            _ => "Forest Walk",
        };
        meditations
            .insert(
                name.to_owned(),
                "audio/ogg",
                1000,
                format!("k/{name}/{i}"),
                1000,
                now,
                None,
            )
            .await
            .unwrap();
    }
    let focus = programs
        .insert("Focus Program".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    let sleep = categories.insert("Sleep".to_owned(), now).await.unwrap();
    let home = programs
        .insert("Home Collection".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    programs.set_category(home, Some(sleep.id)).await.unwrap();
    programs.set_cover_key(home, "covers/home").await.unwrap();
    memberships.add(focus, 3, now).await.unwrap();
    memberships.add(home, 5, now).await.unwrap();
}

pub(super) async fn library_q(app: &TestApp, query: &str) -> Vec<serde_json::Value> {
    let (status, _, body) = call(
        app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/library{query}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    serde_json::from_slice(&body).unwrap()
}

pub(super) async fn seed_meditation(app: &TestApp) -> meditation_domain::Meditation {
    // A ready Ogg-Opus-shaped blob the fakes accept.
    let mut bytes = b"OggS".to_vec();
    bytes.extend_from_slice(b"....OpusHead-payload-for-tests");
    crate::audio_upload::upload_for_tests(&app.state, &app.store, bytes).await
}

/// Builds a multipart body from `(field name, filename, bytes)` triples; an
/// empty filename means a plain text field.
pub(super) fn multipart_body(fields: &[(&str, &str, &[u8])]) -> (String, Vec<u8>) {
    let boundary = "router-tests-boundary-4471";
    let mut body = Vec::new();
    for (name, filename, bytes) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        if filename.is_empty() {
            body.extend_from_slice(
                format!("content-disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
            );
        } else {
            body.extend_from_slice(
                format!(
                    "content-disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
                )
                .as_bytes(),
            );
            body.extend_from_slice(b"content-type: application/octet-stream\r\n\r\n");
        }
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

pub(super) async fn expected_len(_app: &TestApp, _id: i64) -> usize {
    // The fake converter turns non-Opus input into a fixed payload; the
    // seeded upload used an Opus-shaped prefix, so the stored length is the
    // input length. Derive from the store instead of duplicating the magic:
    4 + "....OpusHead-payload-for-tests".len()
}

pub(super) async fn put_description(
    app: &TestApp,
    actor: Option<Actor>,
    id: i64,
    json: &str,
) -> (StatusCode, serde_json::Value) {
    let (status, _, body) = json_body(
        app,
        actor,
        "PUT",
        &format!("/meditation/meditations/{id}"),
        json,
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

pub(super) async fn seed_ordered_program(app: &TestApp) -> i64 {
    use meditation_application::ports::{
        MeditationRepository, MembershipRepository, ProgramRepository,
    };
    let meditations = FakeMeditationRepository(app.state.clone());
    let now = Utc::now();
    for i in 0..3 {
        meditations
            .insert(
                format!("M{i}"),
                "audio/ogg",
                1000,
                format!("k/{i}"),
                1000,
                now,
                None,
            )
            .await
            .unwrap();
    }
    let program = FakeProgramRepository(app.state.clone())
        .insert("P".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    let memberships = FakeMembershipRepository(app.state.clone());
    for id in [1, 2, 3] {
        memberships.add(program, id, now).await.unwrap();
    }
    program
}

pub(super) async fn put_order(
    app: &TestApp,
    actor: Actor,
    program: i64,
    ids: &str,
) -> (StatusCode, serde_json::Value) {
    let (status, _, body) = call(
        app,
        Some(actor),
        "PUT",
        &format!("/meditation/programs/{program}/meditations/order"),
        vec![("content-type", "application/json")],
        Body::from(format!(r#"{{"meditation_ids":{ids}}}"#)),
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

pub(super) async fn member_ids(app: &TestApp, program: i64) -> Vec<i64> {
    let (status, _, body) = call(
        app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/programs/{program}/members"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    rows.iter().map(|row| row["id"].as_i64().unwrap()).collect()
}
