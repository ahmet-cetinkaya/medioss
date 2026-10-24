use super::SearchIndex;
use crate::commands::set_program_category::{
    SetProgramCategory, SetProgramCategoryHandler, SetProgramCategoryService,
};
use crate::ports::CategoryRepository;
use crate::test_support::TestDeps;

async fn build(deps: &TestDeps) -> SearchIndex {
    SearchIndex::build(
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
        deps.categories.as_ref(),
    )
    .await
    .expect("build index")
}

#[tokio::test]
async fn haystack_includes_name_program_title_and_category_name() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Évening Calm").await;
    let meditation = deps.seed_meditation_in(program, "Body Scan").await;
    let category = deps
        .categories
        .insert("Sleep Aids".into(), deps.clock.0)
        .await
        .unwrap();
    SetProgramCategoryService::new(deps.programs.clone(), deps.categories.clone())
        .handle(SetProgramCategory {
            program_id: program,
            category_id: Some(category.id),
        })
        .await
        .unwrap();

    let haystack = build(&deps).await.haystack_for(&meditation);

    assert!(haystack.contains("body scan"));
    assert!(haystack.contains("evening calm"));
    assert!(haystack.contains("sleep aids"));
}

#[tokio::test]
async fn haystack_without_program_is_just_the_name() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    deps.memberships.0.memberships.lock().unwrap().clear();

    let haystack = build(&deps).await.haystack_for(&meditation);

    assert_eq!(haystack, "seed");
}

#[tokio::test]
async fn haystack_of_uncategorised_program_omits_category() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Plain").await;
    let meditation = deps.seed_meditation_in(program, "Sit").await;
    deps.categories
        .insert("Unused".into(), deps.clock.0)
        .await
        .unwrap();

    let haystack = build(&deps).await.haystack_for(&meditation);

    assert_eq!(haystack, "sit plain");
}

#[tokio::test]
async fn meditation_in_two_programs_matches_both_titles() {
    let deps = TestDeps::new();
    let first = deps.seed_program("Alpha").await;
    let second = deps.seed_program("Beta").await;
    let meditation = deps.seed_meditation_in(first, "Sit").await;
    deps.memberships
        .0
        .memberships
        .lock()
        .unwrap()
        .push(crate::ports::Membership {
            program_id: second,
            meditation_id: meditation.id(),
            added_at: deps.clock.0,
            position: 0,
        });

    let haystack = build(&deps).await.haystack_for(&meditation);

    assert!(haystack.contains("alpha"));
    assert!(haystack.contains("beta"));
}
