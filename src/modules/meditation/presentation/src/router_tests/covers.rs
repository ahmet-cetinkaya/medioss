use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use meditation_application::{
    ports::ProgramRepository as _,
    test_support::{
        FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeMeditationRepository,
        FakeMembershipRepository, FakeNoteRepository, FakePlaybackRepository,
        FakeProgramRepository, FakeSessionRepository, FakeValidator, FixedClock, RepoState,
    },
};

use super::helpers::*;
use crate::{meditation_router, MeditationDependencies};

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
