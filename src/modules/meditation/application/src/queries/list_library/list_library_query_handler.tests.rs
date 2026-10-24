use crate::ports::QueryHandler;
use std::sync::Arc;

use chrono::Utc;

use crate::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::queries::list_library::{ListLibraryQuery, ListLibraryQueryHandler};
use crate::test_support::{
    FakeCategoryRepository, FakeMeditationRepository, FakeMembershipRepository,
    FakeProgramRepository, RepoState,
};

type SeededLibrary = (
    Arc<FakeMeditationRepository>,
    Arc<FakeMembershipRepository>,
    Arc<FakeProgramRepository>,
    Arc<FakeCategoryRepository>,
);

async fn seeded_list_library() -> SeededLibrary {
    let meditations = Arc::new(FakeMeditationRepository(RepoState::new()));
    let memberships = Arc::new(FakeMembershipRepository(RepoState::new()));
    let programs = Arc::new(FakeProgramRepository(RepoState::new()));
    let categories = Arc::new(FakeCategoryRepository(RepoState::new()));
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
    let focus_program = programs
        .insert("Focus Program".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    let sleep = categories.insert("Sleep".to_owned(), now).await.unwrap();
    let home_program = programs
        .insert("Home Collection".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    programs
        .set_category(home_program, Some(sleep.id))
        .await
        .unwrap();
    memberships.add(focus_program, 3, now).await.unwrap();
    memberships.add(home_program, 5, now).await.unwrap();
    (meditations, memberships, programs, categories)
}

async fn search_library(seeded: &SeededLibrary, query: Option<&str>) -> Vec<i64> {
    ListLibraryQueryHandler::new(
        seeded.0.clone(),
        seeded.1.clone(),
        seeded.2.clone(),
        seeded.3.clone(),
    )
    .handle(ListLibraryQuery {
        query: query.map(str::to_owned),
    })
    .await
    .unwrap()
    .iter()
    .map(|entry| entry.meditation.id())
    .collect()
}

#[tokio::test]
async fn blank_query_returns_the_full_library() {
    let seeded = seeded_list_library().await;
    for query in [None, Some(""), Some("   ")] {
        let ids = search_library(&seeded, query).await;
        assert_eq!(ids.len(), 120, "query: {query:?}");
    }
}

#[tokio::test]
async fn name_matches_case_and_diacritic_insensitively() {
    let seeded = seeded_list_library().await;
    for query in ["morning", "MORNING", "breath"] {
        let ids = search_library(&seeded, Some(query)).await;
        let expected = if query == "breath" { 40 } else { 20 };
        assert_eq!(ids.len(), expected, "query: {query}");
    }
    let ids = search_library(&seeded, Some("cafe quietude")).await;
    assert_eq!(ids.len(), 20);
}

#[tokio::test]
async fn program_and_category_names_match() {
    let seeded = seeded_list_library().await;
    let ids = search_library(&seeded, Some("focus")).await;
    assert_eq!(ids, vec![3]);
    let ids = search_library(&seeded, Some("sleep")).await;
    assert_eq!(ids, vec![5]);
}

#[tokio::test]
async fn multi_term_requires_every_term() {
    let seeded = seeded_list_library().await;
    let ids = search_library(&seeded, Some("morning breathing")).await;
    assert_eq!(ids.len(), 20);
    let ids = search_library(&seeded, Some("morning dreamscape")).await;
    assert!(ids.is_empty());
}

#[tokio::test]
async fn no_match_returns_an_empty_list() {
    let seeded = seeded_list_library().await;
    let ids = search_library(&seeded, Some("xyzzy")).await;
    assert!(ids.is_empty());
}
