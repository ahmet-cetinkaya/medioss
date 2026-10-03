use crate::models::SearchIndex;
use crate::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::search;
use crate::ApplicationError;

pub async fn list_library<R, M, P, C>(
    repository: &R,
    memberships: &M,
    programs: &P,
    categories: &C,
    query: Option<&str>,
) -> Result<Vec<meditation_domain::Meditation>, ApplicationError>
where
    R: MeditationRepository + ?Sized,
    M: MembershipRepository + ?Sized,
    P: ProgramRepository + ?Sized,
    C: CategoryRepository + ?Sized,
{
    let library = repository
        .list_all()
        .await
        .map_err(ApplicationError::Repository)?;
    let is_blank = query.map(str::trim).is_none_or(str::is_empty);
    if is_blank {
        return Ok(library);
    }
    let query = query.unwrap_or_default();
    let index = SearchIndex::build(memberships, programs, categories).await?;
    Ok(library
        .into_iter()
        .filter(|meditation| search::matches(query, &index.haystack_for(meditation)))
        .collect())
}
