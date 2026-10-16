use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_application::ports::{LoginThrottleRepository, RepositoryError};
use identity_domain::FailedAttempts;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};

use super::timestamps::{parse_rfc3339, to_rfc3339};
use super::SeaOrmUserRepository;
use crate::persistence::login_attempts;

#[async_trait]
impl LoginThrottleRepository for SeaOrmUserRepository {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        let row = login_attempts::Entity::find_by_id(normalized_identifier.to_owned())
            .one(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        match row {
            None => Ok(None),
            Some(model) => {
                let count = u64::try_from(model.failed_attempt_count)
                    .map_err(|_| RepositoryError::OperationFailed)?;
                if count == 0 {
                    return Ok(None);
                }
                let last = model
                    .last_failed_attempt_at
                    .as_deref()
                    .map(parse_rfc3339)
                    .transpose()?
                    .ok_or(RepositoryError::OperationFailed)?;
                let blocked = model
                    .blocked_until
                    .as_deref()
                    .map(parse_rfc3339)
                    .transpose()?
                    .ok_or(RepositoryError::OperationFailed)?;
                FailedAttempts::restore(count, last, blocked)
                    .map(Some)
                    .map_err(|_| RepositoryError::OperationFailed)
            }
        }
    }

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: DateTime<Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        let current = login_attempts::Entity::find_by_id(normalized_identifier.to_owned())
            .one(&txn)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        let (existing, attempts) = match current {
            None => (None, FailedAttempts::first_failure(attempted_at)),
            Some(model) => {
                let count = u64::try_from(model.failed_attempt_count)
                    .map_err(|_| RepositoryError::OperationFailed)?;
                if count == 0 {
                    (Some(model), FailedAttempts::first_failure(attempted_at))
                } else {
                    let last = model
                        .last_failed_attempt_at
                        .as_deref()
                        .map(parse_rfc3339)
                        .transpose()?
                        .ok_or(RepositoryError::OperationFailed)?;
                    let blocked = model
                        .blocked_until
                        .as_deref()
                        .map(parse_rfc3339)
                        .transpose()?
                        .ok_or(RepositoryError::OperationFailed)?;
                    let attempts = FailedAttempts::restore(count, last, blocked)
                        .map_err(|_| RepositoryError::OperationFailed)?
                        .record_failure(attempted_at);
                    (Some(model), attempts)
                }
            }
        };
        match existing {
            Some(_) => {
                login_attempts::Entity::update_many()
                    .col_expr(
                        login_attempts::Column::FailedAttemptCount,
                        Expr::value(
                            i64::try_from(attempts.count())
                                .map_err(|_| RepositoryError::OperationFailed)?,
                        ),
                    )
                    .col_expr(
                        login_attempts::Column::LastFailedAttemptAt,
                        Expr::value(to_rfc3339(&attempts.last_attempt_at())),
                    )
                    .col_expr(
                        login_attempts::Column::BlockedUntil,
                        Expr::value(to_rfc3339(&attempts.blocked_until())),
                    )
                    .filter(
                        login_attempts::Column::NormalizedIdentifier
                            .eq(normalized_identifier.to_owned()),
                    )
                    .exec(&txn)
                    .await
                    .map_err(|_| RepositoryError::OperationFailed)?;
            }
            None => {
                login_attempts::ActiveModel {
                    normalized_identifier: Set(normalized_identifier.to_owned()),
                    failed_attempt_count: Set(i64::try_from(attempts.count())
                        .map_err(|_| RepositoryError::OperationFailed)?),
                    last_failed_attempt_at: Set(Some(to_rfc3339(&attempts.last_attempt_at()))),
                    blocked_until: Set(Some(to_rfc3339(&attempts.blocked_until()))),
                }
                .insert(&txn)
                .await
                .map_err(|_| RepositoryError::OperationFailed)?;
            }
        }
        txn.commit()
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        Ok(attempts)
    }

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        login_attempts::Entity::delete_many()
            .filter(login_attempts::Column::NormalizedIdentifier.eq(normalized_identifier))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|_| RepositoryError::OperationFailed)
    }
}
