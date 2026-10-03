mod daily_suggestion;

use crate::ports::{Clock, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;
use chrono::Datelike;

pub use self::daily_suggestion::DailySuggestion;

pub async fn get_daily_suggestion<R, C, M, P>(
    repository: &R,
    clock: &C,
    memberships: &M,
    programs: &P,
) -> Result<Option<DailySuggestion>, ApplicationError>
where
    R: MeditationRepository + ?Sized,
    C: Clock + ?Sized,
    M: MembershipRepository + ?Sized,
    P: ProgramRepository + ?Sized,
{
    let mut library = repository
        .list_all()
        .await
        .map_err(ApplicationError::Repository)?;
    if library.is_empty() {
        return Ok(None);
    }
    // Stable order regardless of the repository's own ordering, so the
    // day's pick does not shift if newer rows are inserted later today.
    library.sort_by_key(|meditation| meditation.id());
    let day = clock.now().await.num_days_from_ce();
    let index = (day.rem_euclid(library.len() as i32)) as usize;
    let Some(meditation) = library.into_iter().nth(index) else {
        return Ok(None);
    };
    let program = match memberships
        .first_program_for_meditation(meditation.id())
        .await
        .map_err(ApplicationError::Repository)?
    {
        Some(program_id) => programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?,
        None => None,
    };
    Ok(Some(DailySuggestion {
        meditation,
        program,
    }))
}
