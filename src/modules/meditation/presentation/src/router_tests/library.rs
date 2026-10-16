use axum::{body::Body, http::StatusCode};

use super::helpers::*;

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
