//! Router-level tests: authorization gates, streaming Range matrix, and
//! error mapping against fake ports (identity router.tests.rs style).
use std::sync::Arc;

use chrono::{Duration, TimeZone, Utc};

use axum::{
    body::Body,
    http::{header, HeaderName, Request, StatusCode},
    Router,
};
use kernel::{Actor, Role, UserId};
use tower::ServiceExt;

use meditation_application::{
    ports::{ProgramRepository as _, SessionRepository},
    test_support::{
        FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
        FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository,
        FakePlaybackRepository, FakeProgramRepository, FakeSessionRepository, FakeValidator,
        FixedClock, RepoState,
    },
};

use crate::{meditation_router, MeditationDependencies};

fn admin_actor() -> Actor {
    Actor::new(UserId::new(1), Role::Admin)
}

fn member_actor() -> Actor {
    Actor::new(UserId::new(2), Role::Member)
}

struct TestApp {
    router: Router,
    state: RepoState,
    store: FakeBlobStore,
}

fn app() -> TestApp {
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

#[tokio::test]
async fn statistics_returns_only_the_callers_sessions() {
    let test_app = app();
    let sessions = FakeSessionRepository(test_app.state.clone());
    let started_at = Utc
        .with_ymd_and_hms(2026, 9, 29, 8, 0, 0)
        .single()
        .expect("test instant");
    sessions
        .record_if_new(
            UserId::new(2),
            1,
            started_at,
            started_at + Duration::minutes(25),
        )
        .await
        .expect("member session");
    sessions
        .record_if_new(
            UserId::new(1),
            2,
            started_at,
            started_at + Duration::minutes(60),
        )
        .await
        .expect("admin session");

    let (status, _, body) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=all&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let body = String::from_utf8(body).expect("json");
    assert!(body.contains("\"total_minutes\":25"));
    assert!(!body.contains("60"));
}

#[tokio::test]
async fn statistics_returns_401_and_rejects_invalid_query_values() {
    let test_app = app();
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::get("/meditation/statistics?range=all&tz_offset_minutes=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let (status, _, _) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=year&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _, _) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=all&tz_offset_minutes=1441",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

async fn call(
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

async fn json_body(
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

async fn seed_search_library(app: &TestApp) {
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

async fn library_q(app: &TestApp, query: &str) -> Vec<serde_json::Value> {
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

#[tokio::test]
async fn meditation_detail_carries_its_home_program_and_cover_flag() {
    let app = app();
    seed_search_library(&app).await;

    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/5",
        vec![],
        Body::empty(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(response["program_id"], serde_json::json!(2));
    assert_eq!(response["program_has_cover"], serde_json::json!(true));
    assert_eq!(
        response["program"],
        serde_json::json!({"id": 2, "title": "Home Collection", "has_cover": true})
    );

    // The home program's title travels too, cover or not.
    let (_, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/3",
        vec![],
        Body::empty(),
    )
    .await;
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        response["program"],
        serde_json::json!({"id": 1, "title": "Focus Program", "has_cover": false})
    );

    // No membership: no program summary (absent, like program_id).
    let (_, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/1",
        vec![],
        Body::empty(),
    )
    .await;
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response.get("program").is_none());
}

#[tokio::test]
async fn library_search_filters_by_name_program_and_category() {
    let app = app();
    seed_search_library(&app).await;

    let ids: Vec<i64> = library_q(&app, "?q=focus")
        .await
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    assert_eq!(ids, vec![3]);

    let ids: Vec<i64> = library_q(&app, "?q=SLEEP")
        .await
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect();
    assert_eq!(ids, vec![5]);

    let names: Vec<String> = library_q(&app, "?q=morning")
        .await
        .iter()
        .filter_map(|item| item["name"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(names.len(), 20);
    assert!(names.iter().all(|name| name == "Morning Breathing"));

    let cafe: Vec<String> = library_q(&app, "?q=cafe%20quietude")
        .await
        .iter()
        .filter_map(|item| item["name"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(cafe.len(), 20);
}

#[tokio::test]
async fn library_search_blank_or_absent_q_returns_everything() {
    let app = app();
    seed_search_library(&app).await;
    for query in ["", "?q=", "?q=%20%20"] {
        let listed = library_q(&app, query).await;
        assert_eq!(listed.len(), 120, "query: {query}");
    }
}

#[tokio::test]
async fn library_rows_carry_their_program_id_or_none() {
    let app = app();
    seed_search_library(&app).await;
    let listed = library_q(&app, "").await;
    let mut with_program: Vec<i64> = listed
        .iter()
        .filter(|row| row.get("program_id").is_some())
        .map(|row| row["id"].as_i64().unwrap())
        .collect();
    with_program.sort();
    assert_eq!(with_program, vec![3, 5]);
}

#[tokio::test]
async fn library_rows_carry_their_home_program_cover_flag() {
    let app = app();
    seed_search_library(&app).await;
    let listed = library_q(&app, "").await;
    let by_id = |id: i64| {
        listed
            .iter()
            .find(|row| row["id"].as_i64() == Some(id))
            .unwrap_or_else(|| panic!("meditation {id} missing from library"))
    };
    // Home Collection has a cover; Focus Program does not; the rest have no
    // program at all (flag absent via serde skip).
    assert_eq!(by_id(5)["program_id"], serde_json::json!(2));
    assert_eq!(by_id(5)["program_has_cover"], serde_json::json!(true));
    assert_eq!(by_id(3)["program_id"], serde_json::json!(1));
    // Focus Program has no cover: the flag is skipped at `false` (serde
    // convention shared with every other surface).
    assert_eq!(by_id(3)["program_has_cover"], serde_json::json!(null));
    let untouched = by_id(1);
    assert!(untouched.get("program_id").is_none());
    assert_eq!(untouched["program_has_cover"], serde_json::json!(null));
}

#[tokio::test]
async fn library_search_rejects_overlong_query_with_400() {
    let app = app();
    seed_search_library(&app).await;
    let long = "a".repeat(201);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/library?q={long}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let exact = "a".repeat(200);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/library?q={exact}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn library_search_no_match_returns_empty_list() {
    let app = app();
    seed_search_library(&app).await;
    let listed = library_q(&app, "?q=xyzzy").await;
    assert!(listed.is_empty());
}

async fn seed_meditation(app: &TestApp) -> meditation_domain::Meditation {
    // A ready Ogg-Opus-shaped blob the fakes accept.
    let mut bytes = b"OggS".to_vec();
    bytes.extend_from_slice(b"....OpusHead-payload-for-tests");
    crate::audio_upload::upload_for_tests(&app.state, &app.store, bytes).await
}

/// Builds a multipart body from `(field name, filename, bytes)` triples; an
/// empty filename means a plain text field.
fn multipart_body(fields: &[(&str, &str, &[u8])]) -> (String, Vec<u8>) {
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

#[tokio::test]
async fn every_route_requires_an_actor() {
    let app = app();
    // A representative management route, read route, and per-user route.
    for (method, path) in [
        ("POST", "/meditation/audio"),
        ("GET", "/meditation/library"),
        ("GET", "/meditation/1"),
        ("GET", "/meditation/1/audio"),
        ("GET", "/meditation/1/download"),
        ("GET", "/meditation/1/playback-state"),
        ("GET", "/meditation/1/notes"),
        ("GET", "/meditation/programs"),
        ("GET", "/meditation/categories"),
        ("PUT", "/meditation/categories/1"),
    ] {
        let (status, _, _) = call(&app, None, method, path, vec![], Body::empty()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
}

#[tokio::test]
async fn content_management_is_admin_only() {
    let app = app();
    for (method, path) in [
        ("POST", "/meditation/audio"),
        ("DELETE", "/meditation/1"),
        ("POST", "/meditation/programs"),
        ("PUT", "/meditation/programs/1"),
        ("DELETE", "/meditation/programs/1"),
        ("POST", "/meditation/programs/1/members"),
        ("DELETE", "/meditation/programs/1/members/1"),
        ("PUT", "/meditation/programs/1/category"),
        ("PUT", "/meditation/programs/1/meditations/order"),
        ("POST", "/meditation/categories"),
        ("PUT", "/meditation/categories/1"),
        ("DELETE", "/meditation/categories/1"),
    ] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            method,
            path,
            vec![("content-type", "application/json")],
            Body::from("{}"),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
    }
}

#[tokio::test]
async fn unknown_ids_are_404_for_every_authenticated_reader() {
    let app = app();
    for path in [
        "/meditation/999",
        "/meditation/999/audio",
        "/meditation/999/download",
        "/meditation/999/playback-state",
        "/meditation/999/notes",
    ] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            "GET",
            path,
            vec![],
            Body::empty(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "GET {path}");
    }
}

#[tokio::test]
async fn range_matrix_over_stored_audio() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = format!("/meditation/{}/audio", meditation.id());

    // No Range: 200 with the full body.
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.is_empty());
    assert!(headers
        .iter()
        .any(|(name, _)| name == header::ACCEPT_RANGES));

    // bytes=0-99: 206 with exactly 100 bytes (or the whole blob when smaller).
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=0-99")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        body.len(),
        100.min(expected_len(&app, meditation.id()).await)
    );
    let content_range = headers
        .iter()
        .find(|(name, _)| name == header::CONTENT_RANGE)
        .expect("content-range present");
    assert!(
        content_range.1.starts_with("bytes 0-"),
        "clamped end for short blobs: {content_range:?}"
    );

    // Open-ended: bytes=10-
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=10-")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    let total = expected_len(&app, meditation.id()).await as usize;
    assert_eq!(body.len(), total - 10);

    // Suffix: bytes=-5 → the last 5 bytes.
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=-5")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(body.len(), 5);

    // Unsatisfiable: 416 with `bytes */total`.
    let (status, headers, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=999999-")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert!(headers
        .iter()
        .any(|(name, value)| name == header::CONTENT_RANGE && value.starts_with("bytes */")));

    // Malformed: ignored, full 200.
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "not-a-range")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

async fn expected_len(_app: &TestApp, _id: i64) -> usize {
    // The fake converter turns non-Opus input into a fixed payload; the
    // seeded upload used an Opus-shaped prefix, so the stored length is the
    // input length. Derive from the store instead of duplicating the magic:
    4 + "....OpusHead-payload-for-tests".len()
}

#[tokio::test]
async fn download_is_an_attachment_named_after_the_meditation() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, headers, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/download", meditation.id()),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let disposition = headers
        .iter()
        .find(|(name, _)| name == header::CONTENT_DISPOSITION)
        .expect("disposition present");
    assert!(disposition.1.starts_with("attachment; filename="));
    assert!(disposition.1.ends_with(".opus\""));
}

#[tokio::test]
async fn completion_retry_is_idempotent_and_toggle_touches_nothing() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let started = chrono::Utc::now().to_rfc3339();
    let finished = chrono::Utc::now().to_rfc3339();
    let payload = format!("{{\"started_at\":\"{started}\",\"finished_at\":\"{finished}\"}}");

    let (first, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/completion", meditation.id()),
        &payload,
    )
    .await;
    assert_eq!(first, StatusCode::CREATED);
    let (retry, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/completion", meditation.id()),
        &payload,
    )
    .await;
    assert_eq!(retry, StatusCode::CREATED);
    assert_eq!(
        app.state.sessions.lock().unwrap().len(),
        1,
        "one session, not two"
    );

    // Toggle completed off and on: session count stays one.
    let id = meditation.id();
    for completed in [false, true] {
        let (status, _, _) = json_body(
            &app,
            Some(member_actor()),
            "PUT",
            &format!("/meditation/{id}/completed"),
            &format!("{{\"completed\":{completed}}}"),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert_eq!(app.state.sessions.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn notes_of_one_user_are_invisible_to_another() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/notes", meditation.id()),
        "{\"text\":\"private reflection\"}",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // The same member sees it; a different member's list is empty.
    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "{}",
    )
    .await;
    assert!(String::from_utf8_lossy(&body).contains("private reflection"));

    let other = Actor::new(UserId::new(3), Role::Member);
    let (_, _, body) = json_body(
        &app,
        Some(other.clone()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "{}",
    )
    .await;
    assert_eq!(body, b"[]");

    // The other member cannot edit or delete what it cannot see.
    let (status, _, _) = json_body(
        &app,
        Some(other),
        "PUT",
        &format!("/meditation/{}/notes/1", meditation.id()),
        "{\"text\":\"hijack\"}",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn progress_percent_reflects_the_reported_position() {
    let app = app();
    let meditation = seed_meditation(&app).await; // duration_ms == 60_000

    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "{}",
    )
    .await;
    assert!(
        String::from_utf8_lossy(&body).contains("\"progress_percent\":0"),
        "untouched meditation reports 0, never omitted: {}",
        String::from_utf8_lossy(&body)
    );

    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/playback-state", meditation.id()),
        "{\"position_ms\":30000}",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "{}",
    )
    .await;
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("\"progress_percent\":50"),
        "30s of 60s must report 50%: {text}"
    );
    assert!(text.contains("\"completed\":false"));

    // Someone else's position report must not move this meditation's own
    // percent for the original caller.
    let other = Actor::new(UserId::new(3), Role::Member);
    let (_, _, body) = json_body(
        &app,
        Some(other),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "{}",
    )
    .await;
    assert!(String::from_utf8_lossy(&body).contains("\"progress_percent\":0"));
}

#[tokio::test]
async fn replacing_a_cover_removes_the_previous_blob() {
    let app = app();
    let png = vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0, 0, 0, 0];
    let (content_type, body) =
        multipart_body(&[("title", "", b"Covered"), ("cover", "cover.png", &png)]);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/programs",
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let text = String::from_utf8_lossy(&body);
    let start = text.find("\"id\":").map(|index| index + 5).unwrap();
    let end = text[start..].find(',').map(|index| start + index).unwrap();
    let program_id: i64 = text[start..end].parse().unwrap();
    assert_eq!(app.store.len(), 1, "one cover blob after create");

    let jpeg = vec![0xFF, 0xD8, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let (content_type, body) = multipart_body(&[("cover", "new.jpg", &jpeg)]);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        &format!("/meditation/programs/{program_id}/cover"),
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        app.store.len(),
        1,
        "the old cover blob is removed, not left behind alongside the new one"
    );
}

#[tokio::test]
async fn cover_normalization_failures_keep_their_operational_statuses() {
    use meditation_application::ports::CoverNormalizationError;
    use meditation_application::test_support::FailingCoverValidator;

    // Server-side conditions never surface as the 415 reserved for bad media:
    // busy is a retryable 503, a failed worker or store is a 500.
    let cases = [
        (
            CoverNormalizationError::InvalidPayload,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            CoverNormalizationError::OutputTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        (
            CoverNormalizationError::WorkerFailed,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            CoverNormalizationError::Busy,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
    ];
    for (error, expected) in cases {
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
            cover_validator: Arc::new(FailingCoverValidator(error)),
            converter: Arc::new(FakeConverter { fail: false }),
            store: Arc::new(store.clone()),
            clock: Arc::new(FixedClock(chrono::Utc::now())),
            medioss_origin: Arc::new("http://localhost".to_owned()),
        };
        let router = meditation_router().with_state(Arc::new(deps));

        let program_id = FakeProgramRepository(state.clone())
            .insert("Holder".to_owned(), None, chrono::Utc::now())
            .await
            .unwrap()
            .id;
        let (content_type, body) = multipart_body(&[("cover", "cover.png", b"not-an-image")]);
        let mut request = Request::put(format!("/meditation/programs/{program_id}/cover"))
            .header("content-type", content_type)
            .header("origin", "http://localhost")
            .body(Body::from(body))
            .unwrap();
        request.extensions_mut().insert(admin_actor());
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), expected, "{error:?}");
        assert!(store.is_empty());
    }
}

#[tokio::test]
async fn cross_origin_writes_are_rejected() {
    let app = app();
    let request = Request::builder()
        .method("POST")
        .uri("/meditation/categories")
        .header("origin", "http://evil.example")
        .header("content-type", "application/json")
        .extension(admin_actor())
        .body(Body::from("{\"name\":\"X\"}"))
        .unwrap();
    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn put_description(
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

#[tokio::test]
async fn upload_with_description_roundtrips_through_library_and_detail() {
    let app = app();
    let program = FakeProgramRepository(app.state.clone())
        .insert("Holder".to_owned(), None, chrono::Utc::now())
        .await
        .unwrap()
        .id;
    let program_id = program.to_string();
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Described"),
        ("program_id", "", program_id.as_bytes()),
        ("description", "", b"  A gentle start  "),
        ("file", "a.opus", b"OggS....OpusHead"),
    ]);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/audio",
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["description"], "A gentle start");
    let id = created["id"].as_i64().unwrap();

    let (_, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(detail["description"], "A gentle start");
    let listed = library_q(&app, "").await;
    assert_eq!(listed[0]["description"], "A gentle start");
}

#[tokio::test]
async fn meditation_without_description_serializes_null() {
    let app = app();
    let seeded = seed_meditation(&app).await;
    let (_, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", seeded.id()),
        vec![],
        Body::empty(),
    )
    .await;
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(detail["description"].is_null());
}

#[tokio::test]
async fn updating_the_description_sets_then_clears_it() {
    let app = app();
    let id = seed_meditation(&app).await.id();

    let (status, body) = put_description(
        &app,
        Some(admin_actor()),
        id,
        r#"{"description":"New text"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["description"], "New text");
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Seed");

    for clear in [r#"{"description":null}"#, r#"{"description":""}"#] {
        put_description(&app, Some(admin_actor()), id, r#"{"description":"x"}"#).await;
        let (status, body) = put_description(&app, Some(admin_actor()), id, clear).await;
        assert_eq!(status, StatusCode::OK, "{clear}");
        assert!(body["description"].is_null(), "{clear}");
    }
}

#[tokio::test]
async fn too_long_description_is_400_with_the_exact_message() {
    let app = app();
    let id = seed_meditation(&app).await.id();
    let long = format!(r#"{{"description":"{}"}}"#, "a".repeat(2001));
    let (status, body) = put_description(&app, Some(admin_actor()), id, &long).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "description must be at most 2000 characters");

    let exact = format!(r#"{{"description":"{}"}}"#, "a".repeat(2000));
    let (status, _) = put_description(&app, Some(admin_actor()), id, &exact).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn description_update_covers_404_403_and_401() {
    let app = app();
    let id = seed_meditation(&app).await.id();
    let (status, _) =
        put_description(&app, Some(admin_actor()), 9999, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) =
        put_description(&app, Some(member_actor()), id, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = put_description(&app, None, id, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

async fn seed_ordered_program(app: &TestApp) -> i64 {
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

async fn put_order(
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

async fn member_ids(app: &TestApp, program: i64) -> Vec<i64> {
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

#[tokio::test]
async fn reorder_roundtrip_changes_member_listing() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    assert_eq!(member_ids(&app, program).await, vec![1, 2, 3]);
    let (status, body) = put_order(&app, admin_actor(), program, "[3,1,2]").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        serde_json::json!({"program_id": program, "meditation_ids": [3, 1, 2]})
    );
    assert_eq!(member_ids(&app, program).await, vec![3, 1, 2]);
}

#[tokio::test]
async fn reorder_rejects_any_set_mismatch() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    for ids in [
        "[1,2]",
        "[1,2,3,3]",
        "[1,1,2]",
        "[1,2,99]",
        "[1,2,3,4]",
        "[]",
    ] {
        let (status, body) = put_order(&app, admin_actor(), program, ids).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{ids}");
        assert_eq!(
            body["error"],
            "meditation_ids must contain exactly the program's meditations"
        );
    }
    assert_eq!(member_ids(&app, program).await, vec![1, 2, 3]);
}

#[tokio::test]
async fn reorder_unknown_program_is_404_and_member_is_403() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    let (status, _) = put_order(&app, admin_actor(), 999, "[1]").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = put_order(&app, member_actor(), program, "[3,2,1]").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(member_ids(&app, program).await, vec![1, 2, 3]);
}
