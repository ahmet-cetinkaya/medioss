use crate::ports::CategoryRepository;
use crate::ApplicationError;

pub(crate) async fn ensure_category_exists(
    categories: &dyn CategoryRepository,
    category_id: i64,
) -> Result<(), ApplicationError> {
    categories
        .find(category_id)
        .await
        .map_err(ApplicationError::Repository)?
        .map(|_| ())
        .ok_or(ApplicationError::NotFound)
}
