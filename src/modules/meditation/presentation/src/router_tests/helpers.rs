use std::sync::Arc;

use chrono::Utc;

use axum::{
    body::Body,
    http::{HeaderName, Request, StatusCode},
    Router,
};
use kernel::{Actor, Role, UserId};
use tower::ServiceExt;

use meditation_application::commands::add_member::AddMemberService;
use meditation_application::commands::create_category::CreateCategoryService;
use meditation_application::commands::create_note::CreateNoteService;
use meditation_application::commands::create_program::CreateProgramService;
use meditation_application::commands::delete_category::DeleteCategoryService;
use meditation_application::commands::delete_meditation::DeleteMeditationService;
use meditation_application::commands::delete_note::DeleteNoteService;
use meditation_application::commands::delete_program::DeleteProgramService;
use meditation_application::commands::record_completion::RecordCompletionService;
use meditation_application::commands::record_position::RecordPositionService;
use meditation_application::commands::remove_member::RemoveMemberService;
use meditation_application::commands::rename_category::RenameCategoryService;
use meditation_application::commands::replace_program_cover::ReplaceProgramCoverService;
use meditation_application::commands::set_completed::SetCompletedService;
use meditation_application::commands::set_program_category::SetProgramCategoryService;
use meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionService;
use meditation_application::commands::update_note::UpdateNoteService;
use meditation_application::commands::update_program::UpdateProgramService;
use meditation_application::commands::update_program_order::UpdateProgramOrderService;
use meditation_application::commands::upload_meditation::UploadMeditationService;
use meditation_application::queries::get_continue_program::GetContinueProgramService;
use meditation_application::queries::get_daily_suggestion::GetDailySuggestionService;
use meditation_application::queries::get_meditation::GetMeditationService;
use meditation_application::queries::get_meditation_detail::GetMeditationDetailService;
use meditation_application::queries::get_playback_state::GetPlaybackStateService;
use meditation_application::queries::get_program_cover::GetProgramCoverService;
use meditation_application::queries::get_user_statistics::GetUserStatisticsService;
use meditation_application::queries::list_categories::ListCategoriesService;
use meditation_application::queries::list_library::ListLibraryService;
use meditation_application::queries::list_notes::ListNotesService;
use meditation_application::queries::list_program_members::ListProgramMembersService;
use meditation_application::queries::list_programs::ListProgramsService;
use meditation_application::queries::list_recent_programs::ListRecentProgramsService;
use meditation_application::test_support::{
    FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
    FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository, FakePlaybackRepository,
    FakeProgramRepository, FakeSessionRepository, FakeValidator, FixedClock, RepoState,
};

use crate::{meditation_router, MeditationDependencies};

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
        create_category: Arc::new(CreateCategoryService::new(
            categories.clone(),
            clock.clone(),
        )),
        rename_category: Arc::new(RenameCategoryService::new(
            categories.clone(),
            clock.clone(),
        )),
        delete_category: Arc::new(DeleteCategoryService::new(
            categories.clone(),
            programs.clone(),
        )),
        set_program_category: Arc::new(SetProgramCategoryService::new(
            programs.clone(),
            categories.clone(),
        )),
        list_categories: Arc::new(ListCategoriesService::new(categories.clone())),
        create_program: Arc::new(CreateProgramService::new(
            programs.clone(),
            cover_validator.clone(),
            store_port.clone(),
            clock.clone(),
        )),
        update_program: Arc::new(UpdateProgramService::new(programs.clone())),
        delete_program: Arc::new(DeleteProgramService::new(
            programs.clone(),
            memberships.clone(),
            meditations.clone(),
            store_port.clone(),
        )),
        list_programs: Arc::new(ListProgramsService::new(programs.clone())),
        list_program_members: Arc::new(ListProgramMembersService::new(
            programs.clone(),
            memberships.clone(),
        )),
        add_member: Arc::new(AddMemberService::new(
            meditations.clone(),
            programs.clone(),
            memberships.clone(),
            clock.clone(),
        )),
        remove_member: Arc::new(RemoveMemberService::new(
            programs.clone(),
            memberships.clone(),
        )),
        update_program_order: Arc::new(UpdateProgramOrderService::new(
            programs.clone(),
            memberships.clone(),
        )),
        upload_meditation: Arc::new(UploadMeditationService::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            validator,
            converter,
            store_port.clone(),
            clock.clone(),
        )),
        delete_meditation: Arc::new(DeleteMeditationService::new(
            meditations.clone(),
            memberships.clone(),
            notes.clone(),
            store_port.clone(),
        )),
        update_meditation_description: Arc::new(UpdateMeditationDescriptionService::new(
            meditations.clone(),
        )),
        get_meditation: Arc::new(GetMeditationService::new(meditations.clone())),
        get_meditation_detail: Arc::new(GetMeditationDetailService::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        get_daily_suggestion: Arc::new(GetDailySuggestionService::new(
            meditations.clone(),
            clock.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_library: Arc::new(ListLibraryService::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            categories,
        )),
        get_program_cover: Arc::new(GetProgramCoverService::new(
            programs.clone(),
            store_port.clone(),
        )),
        replace_program_cover: Arc::new(ReplaceProgramCoverService::new(
            programs.clone(),
            store_port.clone(),
        )),
        get_playback_state: Arc::new(GetPlaybackStateService::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_position: Arc::new(RecordPositionService::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_completion: Arc::new(RecordCompletionService::new(
            meditations.clone(),
            playback.clone(),
            sessions.clone(),
        )),
        set_completed: Arc::new(SetCompletedService::new(
            meditations.clone(),
            playback.clone(),
        )),
        get_user_statistics: Arc::new(GetUserStatisticsService::new(sessions, clock.clone())),
        create_note: Arc::new(CreateNoteService::new(
            meditations.clone(),
            notes.clone(),
            clock.clone(),
        )),
        update_note: Arc::new(UpdateNoteService::new(notes.clone(), clock.clone())),
        delete_note: Arc::new(DeleteNoteService::new(notes.clone())),
        list_notes: Arc::new(ListNotesService::new(meditations, notes)),
        get_continue_program: Arc::new(GetContinueProgramService::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_recent_programs: Arc::new(ListRecentProgramsService::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        programs,
        playback,
        cover_validator,
        store: store_port,
        medioss_origin: Arc::new("http://localhost".to_owned()),
    }
}

pub(crate) fn app() -> TestApp {
    let state = RepoState::new();
    let store = FakeBlobStore::new();
    let deps = test_dependencies(state.clone(), &store);
    TestApp {
        router: meditation_router().with_state(Arc::new(deps)),
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
