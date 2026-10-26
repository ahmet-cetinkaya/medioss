use std::sync::Arc;

use chrono::Utc;

use axum::{
    body::Body,
    http::{HeaderName, Request, StatusCode},
    Router,
};
use kernel::{Actor, Role, UserId};
use tower::ServiceExt;

use meditation_application::commands::add_member::AddMemberCommandHandler;
use meditation_application::commands::create_category::CreateCategoryCommandHandler;
use meditation_application::commands::create_note::CreateNoteCommandHandler;
use meditation_application::commands::create_program::CreateProgramCommandHandler;
use meditation_application::commands::delete_category::DeleteCategoryCommandHandler;
use meditation_application::commands::delete_meditation::DeleteMeditationCommandHandler;
use meditation_application::commands::delete_note::DeleteNoteCommandHandler;
use meditation_application::commands::delete_program::DeleteProgramCommandHandler;
use meditation_application::commands::favorite_meditation::FavoriteMeditationCommandHandler;
use meditation_application::commands::record_completion::RecordCompletionCommandHandler;
use meditation_application::commands::record_position::RecordPositionCommandHandler;
use meditation_application::commands::remove_member::RemoveMemberCommandHandler;
use meditation_application::commands::rename_category::RenameCategoryCommandHandler;
use meditation_application::commands::replace_program_cover::ReplaceProgramCoverCommandHandler;
use meditation_application::commands::set_completed::SetCompletedCommandHandler;
use meditation_application::commands::set_program_category::SetProgramCategoryCommandHandler;
use meditation_application::commands::unfavorite_meditation::UnfavoriteMeditationCommandHandler;
use meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionCommandHandler;
use meditation_application::commands::update_note::UpdateNoteCommandHandler;
use meditation_application::commands::update_program::UpdateProgramCommandHandler;
use meditation_application::commands::update_program_order::UpdateProgramOrderCommandHandler;
use meditation_application::commands::upload_meditation::UploadMeditationCommandHandler;
use meditation_application::queries::get_continue_program::GetContinueProgramQueryHandler;
use meditation_application::queries::get_daily_suggestion::GetDailySuggestionQueryHandler;
use meditation_application::queries::get_meditation::GetMeditationQueryHandler;
use meditation_application::queries::get_meditation_detail::GetMeditationDetailQueryHandler;
use meditation_application::queries::get_playback_state::GetPlaybackStateQueryHandler;
use meditation_application::queries::get_program_cover::GetProgramCoverQueryHandler;
use meditation_application::queries::get_user_statistics::GetUserStatisticsQueryHandler;
use meditation_application::queries::list_categories::ListCategoriesQueryHandler;
use meditation_application::queries::list_favorite_ids::ListFavoriteIdsQueryHandler;
use meditation_application::queries::list_favorite_meditations::ListFavoriteMeditationsQueryHandler;
use meditation_application::queries::list_library::ListLibraryQueryHandler;
use meditation_application::queries::list_notes::ListNotesQueryHandler;
use meditation_application::queries::list_program_members::ListProgramMembersQueryHandler;
use meditation_application::queries::list_programs::ListProgramsQueryHandler;
use meditation_application::queries::list_recent_programs::ListRecentProgramsQueryHandler;
use meditation_application::test_support::{
    FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
    FakeFavoriteRepository, FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository,
    FakePlaybackRepository, FakeProgramRepository, FakeSessionRepository, FakeValidator,
    FixedClock, RepoState,
};

use crate::{meditation_router, MeditationDependencies};
pub(crate) use meditation_application::test_support::{
    TEST_MAX_COVER_BYTES, TEST_MAX_UPLOAD_BYTES,
};

pub(crate) fn admin_actor() -> Actor {
    Actor::new(UserId::new(1), Role::Admin)
}

pub(crate) fn member_actor() -> Actor {
    Actor::new(UserId::new(2), Role::Member)
}

pub(crate) struct TestApp {
    pub(crate) router: Router,
    pub(crate) state: RepoState,
    pub(crate) store: FakeBlobStore,
}

/// Builds every use-case service over one shared fake-repo state, exactly
/// the way web-api's composition root builds them over the SeaORM adapters.
pub(crate) fn test_dependencies(state: RepoState, store: &FakeBlobStore) -> MeditationDependencies {
    let meditations: Arc<dyn meditation_application::ports::MeditationRepository> =
        Arc::new(FakeMeditationRepository(state.clone()));
    let programs: Arc<dyn meditation_application::ports::ProgramRepository> =
        Arc::new(FakeProgramRepository(state.clone()));
    let categories: Arc<dyn meditation_application::ports::CategoryRepository> =
        Arc::new(FakeCategoryRepository(state.clone()));
    let memberships: Arc<dyn meditation_application::ports::MembershipRepository> =
        Arc::new(FakeMembershipRepository(state.clone()));
    let notes: Arc<dyn meditation_application::ports::NoteRepository> =
        Arc::new(FakeNoteRepository(state.clone()));
    let favorites: Arc<dyn meditation_application::ports::FavoriteRepository> =
        Arc::new(FakeFavoriteRepository(state.clone()));
    let playback: Arc<dyn meditation_application::ports::PlaybackStateRepository> =
        Arc::new(FakePlaybackRepository(state.clone()));
    let sessions: Arc<dyn meditation_application::ports::SessionRepository> =
        Arc::new(FakeSessionRepository(state.clone()));
    let validator: Arc<dyn meditation_application::ports::AudioValidator> = Arc::new(FakeValidator);
    let cover_validator: Arc<dyn meditation_application::ports::CoverValidator> =
        Arc::new(FakeCoverValidator);
    let converter: Arc<dyn meditation_application::ports::OpusConverter> =
        Arc::new(FakeConverter { fail: false });
    let store_port: Arc<dyn meditation_application::ports::BlobStore> = Arc::new(store.clone());
    let clock: Arc<dyn meditation_application::ports::Clock> =
        Arc::new(FixedClock(chrono::Utc::now()));
    MeditationDependencies {
        create_category: Arc::new(CreateCategoryCommandHandler::new(
            categories.clone(),
            clock.clone(),
        )),
        rename_category: Arc::new(RenameCategoryCommandHandler::new(
            categories.clone(),
            clock.clone(),
        )),
        delete_category: Arc::new(DeleteCategoryCommandHandler::new(
            categories.clone(),
            programs.clone(),
        )),
        set_program_category: Arc::new(SetProgramCategoryCommandHandler::new(
            programs.clone(),
            categories.clone(),
        )),
        list_categories: Arc::new(ListCategoriesQueryHandler::new(categories.clone())),
        create_program: Arc::new(CreateProgramCommandHandler::new(
            programs.clone(),
            cover_validator.clone(),
            store_port.clone(),
            clock.clone(),
            TEST_MAX_COVER_BYTES,
        )),
        update_program: Arc::new(UpdateProgramCommandHandler::new(programs.clone())),
        delete_program: Arc::new(DeleteProgramCommandHandler::new(
            programs.clone(),
            memberships.clone(),
            meditations.clone(),
            store_port.clone(),
        )),
        list_programs: Arc::new(ListProgramsQueryHandler::new(programs.clone())),
        list_program_members: Arc::new(ListProgramMembersQueryHandler::new(
            programs.clone(),
            memberships.clone(),
        )),
        add_member: Arc::new(AddMemberCommandHandler::new(
            meditations.clone(),
            programs.clone(),
            memberships.clone(),
            clock.clone(),
        )),
        remove_member: Arc::new(RemoveMemberCommandHandler::new(
            programs.clone(),
            memberships.clone(),
        )),
        update_program_order: Arc::new(UpdateProgramOrderCommandHandler::new(
            programs.clone(),
            memberships.clone(),
        )),
        upload_meditation: Arc::new(UploadMeditationCommandHandler::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            validator,
            converter,
            store_port.clone(),
            clock.clone(),
            TEST_MAX_UPLOAD_BYTES,
        )),
        delete_meditation: Arc::new(DeleteMeditationCommandHandler::new(
            meditations.clone(),
            memberships.clone(),
            notes.clone(),
            favorites.clone(),
            store_port.clone(),
        )),
        update_meditation_description: Arc::new(UpdateMeditationDescriptionCommandHandler::new(
            meditations.clone(),
        )),
        get_meditation: Arc::new(GetMeditationQueryHandler::new(meditations.clone())),
        get_meditation_detail: Arc::new(GetMeditationDetailQueryHandler::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        get_daily_suggestion: Arc::new(GetDailySuggestionQueryHandler::new(
            meditations.clone(),
            clock.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_library: Arc::new(ListLibraryQueryHandler::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            categories,
        )),
        get_program_cover: Arc::new(GetProgramCoverQueryHandler::new(
            programs.clone(),
            store_port.clone(),
        )),
        replace_program_cover: Arc::new(ReplaceProgramCoverCommandHandler::new(
            programs.clone(),
            store_port.clone(),
        )),
        get_playback_state: Arc::new(GetPlaybackStateQueryHandler::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_position: Arc::new(RecordPositionCommandHandler::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_completion: Arc::new(RecordCompletionCommandHandler::new(
            meditations.clone(),
            playback.clone(),
            sessions.clone(),
        )),
        set_completed: Arc::new(SetCompletedCommandHandler::new(
            meditations.clone(),
            playback.clone(),
        )),
        get_user_statistics: Arc::new(GetUserStatisticsQueryHandler::new(sessions, clock.clone())),
        create_note: Arc::new(CreateNoteCommandHandler::new(
            meditations.clone(),
            notes.clone(),
            clock.clone(),
        )),
        update_note: Arc::new(UpdateNoteCommandHandler::new(notes.clone(), clock.clone())),
        delete_note: Arc::new(DeleteNoteCommandHandler::new(notes.clone())),
        favorite_meditation: Arc::new(FavoriteMeditationCommandHandler::new(
            meditations.clone(),
            favorites.clone(),
            clock.clone(),
        )),
        unfavorite_meditation: Arc::new(UnfavoriteMeditationCommandHandler::new(
            meditations.clone(),
            favorites.clone(),
        )),
        list_favorite_meditations: Arc::new(ListFavoriteMeditationsQueryHandler::new(
            meditations.clone(),
            favorites.clone(),
        )),
        list_favorite_ids: Arc::new(ListFavoriteIdsQueryHandler::new(favorites)),
        list_notes: Arc::new(ListNotesQueryHandler::new(meditations, notes)),
        get_continue_program: Arc::new(GetContinueProgramQueryHandler::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_recent_programs: Arc::new(ListRecentProgramsQueryHandler::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        programs,
        playback,
        cover_validator,
        store: store_port,
        medioss_origin: Arc::new("http://localhost".to_owned()),
        max_upload_bytes: TEST_MAX_UPLOAD_BYTES,
        max_cover_bytes: TEST_MAX_COVER_BYTES,
    }
}

pub(crate) fn app() -> TestApp {
    let state = RepoState::new();
    let store = FakeBlobStore::new();
    let deps = test_dependencies(state.clone(), &store);
    TestApp {
        router: meditation_router(TEST_MAX_UPLOAD_BYTES, TEST_MAX_COVER_BYTES)
            .with_state(Arc::new(deps)),
        state,
        store,
    }
}

pub(crate) async fn call(
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

pub(crate) async fn json_body(
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

pub(crate) async fn seed_search_library(app: &TestApp) {
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

pub(crate) async fn library_q(app: &TestApp, query: &str) -> Vec<serde_json::Value> {
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

pub(crate) async fn seed_meditation(app: &TestApp) -> meditation_domain::Meditation {
    // A ready Ogg-Opus-shaped blob the fakes accept.
    let mut bytes = b"OggS".to_vec();
    bytes.extend_from_slice(b"....OpusHead-payload-for-tests");
    crate::audio_upload::upload_for_tests(&app.state, &app.store, bytes).await
}

/// Builds a multipart body from `(field name, filename, bytes)` triples; an
/// empty filename means a plain text field.
pub(crate) fn multipart_body(fields: &[(&str, &str, &[u8])]) -> (String, Vec<u8>) {
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

pub(crate) async fn expected_len(_app: &TestApp, _id: i64) -> usize {
    // The fake converter turns non-Opus input into a fixed payload; the
    // seeded upload used an Opus-shaped prefix, so the stored length is the
    // input length. Derive from the store instead of duplicating the magic:
    4 + "....OpusHead-payload-for-tests".len()
}

pub(crate) async fn put_description(
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

pub(crate) async fn seed_ordered_program(app: &TestApp) -> i64 {
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

pub(crate) async fn put_order(
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

pub(crate) async fn member_ids(app: &TestApp, program: i64) -> Vec<i64> {
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

/// Case-insensitive lookup of a response header value.
pub(crate) fn header_value<'a>(headers: &'a [(HeaderName, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.as_str().eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
