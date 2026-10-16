use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use super::helpers::*;

#[tokio::test]
async fn avatar_upload_normalizes_persists_and_replaces() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        store.clone(),
    );
    let upload = || {
        let mut request = Request::put("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .header("content-type", "multipart/form-data; boundary=b")
            .body(Body::from(
                "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nimg\r\n--b--\r\n".to_owned(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        request
    };

    let response = router.clone().oneshot(upload()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = router.clone().oneshot(upload()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    assert_eq!(*store.written.lock().unwrap(), vec!["webp", "webp"]);
    // The second upload stages a fresh key and unlinks the first avatar.
    assert_eq!(*store.removed.lock().unwrap(), vec!["webp"]);
}

#[tokio::test]
async fn avatar_upload_maps_truncated_multipart_to_400() {
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        Arc::new(RecordingAvatarStore::accepting()),
    );
    let mut request = Request::put("/identity/me/avatar")
        .header("origin", "https://medioss.example")
        .header("content-type", "multipart/form-data; boundary=b")
        .body(Body::from(
            "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nimg"
                .to_owned(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn avatar_upload_rejects_wrong_or_missing_multipart_field_with_400() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        store.clone(),
    );

    for field in ["name=\"not_file\"", "filename=\"a\""] {
        let mut request = Request::put("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .header("content-type", "multipart/form-data; boundary=b")
            .body(Body::from(format!(
                "--b\r\ncontent-disposition: form-data; {field}\r\n\r\nimg\r\n--b--\r\n"
            )))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "field: {field}");
    }
    assert!(store.written.lock().unwrap().is_empty());
}

#[tokio::test]
async fn avatar_upload_rejects_invalid_payloads_without_writing() {
    let store = Arc::new(RecordingAvatarStore {
        normalize_result: std::sync::Mutex::new(Err(
            identity_application::ports::AvatarStoreError::InvalidPayload,
        )),
        written: std::sync::Mutex::new(Vec::new()),
        removed: std::sync::Mutex::new(Vec::new()),
        read_result: std::sync::Mutex::new(Vec::new()),
        next_key: std::sync::atomic::AtomicU64::new(1),
    });
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        store.clone(),
    );

    let mut request = Request::put("/identity/me/avatar")
        .header("origin", "https://medioss.example")
        .header("content-type", "multipart/form-data; boundary=b")
        .body(Body::from(
            "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nnot-an-image\r\n--b--\r\n".to_owned(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(store.written.lock().unwrap().is_empty());
}

#[tokio::test]
async fn avatar_upload_maps_normalization_failures_to_operational_statuses() {
    use identity_application::ports::AvatarStoreError;
    // Only invalid input stays a client error; every server-side failure
    // must surface as 5xx, never 400.
    let cases = [
        (
            AvatarStoreError::OutputTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        (
            AvatarStoreError::WorkerFailed,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (AvatarStoreError::Storage, StatusCode::INTERNAL_SERVER_ERROR),
        (AvatarStoreError::Busy, StatusCode::SERVICE_UNAVAILABLE),
    ];
    for (error, expected) in cases {
        let store = Arc::new(RecordingAvatarStore {
            normalize_result: std::sync::Mutex::new(Err(error)),
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            read_result: std::sync::Mutex::new(Vec::new()),
            next_key: std::sync::atomic::AtomicU64::new(1),
        });
        let router = seeded_app_with(
            vec![user(1, "member@example.com", Role::Member)],
            store.clone(),
        );

        let mut request = Request::put("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .header("content-type", "multipart/form-data; boundary=b")
            .body(Body::from(
                "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nimg\r\n--b--\r\n".to_owned(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), expected, "{error:?}");
        assert!(store.written.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn avatar_serving_is_owner_or_admin_only() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let users = vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member)
            .with_avatar(Some(identity_domain::AvatarRef::Webp)),
        user(3, "other@example.com", Role::Member),
    ];
    let router = seeded_app_with(users, store);
    let get = |actor: kernel::Actor| {
        let mut request = Request::get("/identity/avatar/2")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(actor);
        request
    };

    let response = router
        .clone()
        .oneshot(get(kernel::Actor::new(
            kernel::UserId::new(2),
            Role::Member,
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = router
        .clone()
        .oneshot(get(kernel::Actor::new(kernel::UserId::new(1), Role::Admin)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = router
        .clone()
        .oneshot(get(kernel::Actor::new(
            kernel::UserId::new(3),
            Role::Member,
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = router
        .oneshot(
            Request::get("/identity/avatar/2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn avatar_responses_forbid_content_type_sniffing() {
    for (avatar, content_type) in [
        (identity_domain::AvatarRef::Webp, "image/webp"),
        (identity_domain::AvatarRef::Png, "image/png"),
    ] {
        let store = Arc::new(RecordingAvatarStore::accepting());
        let router = seeded_app_with(
            vec![user(2, "member@example.com", Role::Member).with_avatar(Some(avatar))],
            store,
        );
        let mut request = Request::get("/identity/avatar/2")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], content_type);
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    }
}

#[tokio::test]
async fn avatar_delete_is_idempotent() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let users = vec![user(1, "member@example.com", Role::Member)
        .with_avatar(Some(identity_domain::AvatarRef::Webp))];
    let router = seeded_app_with(users, store);
    let delete = || {
        let mut request = Request::delete("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        request
    };

    let response = router.clone().oneshot(delete()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = router.oneshot(delete()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
