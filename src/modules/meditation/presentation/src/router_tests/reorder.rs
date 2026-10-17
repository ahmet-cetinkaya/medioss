use axum::http::StatusCode;

use super::helpers::*;

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
