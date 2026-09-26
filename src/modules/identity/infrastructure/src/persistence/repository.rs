use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_application::ports::{
    CredentialChange, InitialAdminRegistration, LoginThrottleRepository, MemberRegistration,
    RepositoryError, UserRepository,
};
use identity_domain::{Email, FailedAttempts, PasswordHash, Role, User};
use kernel::UserId;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use sea_orm::sea_query::Expr;

use crate::persistence::{login_attempts, users};

pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

const SETUP_CLAIM_SQL: &str = "DELETE FROM instance_state WHERE singleton = 1";

const ROLE_ADMIN: &str = "admin";
const ROLE_MEMBER: &str = "member";

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Admin => ROLE_ADMIN,
        Role::Member => ROLE_MEMBER,
    }
}

fn role_from_str(raw: &str) -> Result<Role, RepositoryError> {
    match raw {
        ROLE_ADMIN => Ok(Role::Admin),
        ROLE_MEMBER => Ok(Role::Member),
        _ => Err(RepositoryError::OperationFailed),
    }
}

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>, RepositoryError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| RepositoryError::OperationFailed)
}

fn to_rfc3339(value: &DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn restore_user(model: users::Model) -> Result<User, RepositoryError> {
    let email = Email::parse(&model.email).map_err(|_| RepositoryError::OperationFailed)?;
    let role = role_from_str(&model.role)?;
    let must_change = model.must_change_password;
    let expires = match (&model.password_expires_at, must_change) {
        (Some(raw), true) => Some(parse_rfc3339(raw)?),
        (None, false) => None,
        _ => return Err(RepositoryError::OperationFailed),
    };
    User::restore(
        UserId::new(model.id),
        email,
        PasswordHash::from_stored(model.password_hash),
        role,
        parse_rfc3339(&model.created_at)?,
        model.auth_version.unsigned_abs(),
        must_change,
        expires,
    )
    .map_err(|_| RepositoryError::OperationFailed)
}

fn insert_model(user: &User, email_raw: &str) -> users::ActiveModel {
    users::ActiveModel {
        email: Set(email_raw.to_owned()),
        password_hash: Set(user.password_hash().as_str().to_owned()),
        role: Set(role_to_str(user.role()).to_owned()),
        must_change_password: Set(user.must_change_password()),
        auth_version: Set(user.auth_version() as i64),
        password_expires_at: Set(user.password_expires_at().map(|value| to_rfc3339(&value))),
        failed_attempt_count: Set(0),
        last_failed_attempt_at: Set(None),
        blocked_until: Set(None),
        created_at: Set(to_rfc3339(&user.created_at())),
        ..Default::default()
    }
}

fn is_unique_violation(error: &sea_orm::DbErr) -> bool {
    match error {
        sea_orm::DbErr::Exec(runtime) | sea_orm::DbErr::Query(runtime) => {
            runtime.to_string().contains("UNIQUE constraint failed")
        }
        _ => false,
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        users::Entity::find()
            .count(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        users::Entity::find()
            .filter(users::Column::Email.eq(email.as_str()))
            .one(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?
            .map(restore_user)
            .transpose()
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        let claimed = txn
            .execute_unprepared(SETUP_CLAIM_SQL)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        if claimed.rows_affected() == 0 {
            txn.rollback()
                .await
                .map_err(|_| RepositoryError::OperationFailed)?;
            return Err(RepositoryError::SetupAlreadyCompleted);
        }
        let user = User::new_permanent(
            UserId::new(0),
            registration.email().clone(),
            registration.password_hash().clone(),
            Role::Admin,
            registration.created_at(),
        );
        let model = insert_model(&user, registration.email().as_str());
        let inserted = model
            .insert(&txn)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        txn.commit()
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        restore_user(inserted)
    }

    async fn insert_member(&self, registration: MemberRegistration) -> Result<User, RepositoryError> {
        let user = User::new_temporary(
            UserId::new(0),
            registration.email().clone(),
            registration.password_hash().clone(),
            Role::Member,
            registration.created_at(),
        )
        .map_err(|_| RepositoryError::OperationFailed)?;
        let model = insert_model(&user, registration.email().as_str());
        let inserted = model
            .insert(&self.db)
            .await
            .map_err(|error| {
                if is_unique_violation(&error) {
                    RepositoryError::EmailAlreadyExists
                } else {
                    RepositoryError::OperationFailed
                }
            })?;
        restore_user(inserted)
    }

    async fn apply_credential_change(
        &self,
        credential_change: CredentialChange,
    ) -> Result<User, RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        let model = users::Entity::find_by_id(credential_change.user_id().value())
            .one(&txn)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?
            .ok_or(RepositoryError::UserNotFound)?;
        let current = restore_user(model.clone())?;
        let changed = current
            .replace_with_permanent_password(credential_change.password_hash().clone())
            .map_err(|_| RepositoryError::AuthVersionOverflow)?;
        login_attempts::Entity::delete_many()
            .filter(login_attempts::Column::NormalizedIdentifier.eq(changed.email().as_str()))
            .exec(&txn)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        users::Entity::update_many()
            .col_expr(
                users::Column::PasswordHash,
                Expr::value(changed.password_hash().as_str().to_owned()),
            )
            .col_expr(
                users::Column::AuthVersion,
                Expr::value(changed.auth_version() as i64),
            )
            .col_expr(users::Column::MustChangePassword, Expr::value(false))
            .col_expr(
                users::Column::PasswordExpiresAt,
                Expr::value(Option::<String>::None),
            )
            .col_expr(users::Column::FailedAttemptCount, Expr::value(0i64))
            .col_expr(
                users::Column::LastFailedAttemptAt,
                Expr::value(Option::<String>::None),
            )
            .col_expr(
                users::Column::BlockedUntil,
                Expr::value(Option::<String>::None),
            )
            .filter(users::Column::Id.eq(credential_change.user_id().value()))
            .exec(&txn)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        txn.commit()
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        let updated = users::Entity::find_by_id(credential_change.user_id().value())
            .one(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?
            .ok_or(RepositoryError::UserNotFound)?;
        restore_user(updated)
    }
}

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
                let count = model.failed_attempt_count.unsigned_abs();
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
                let count = model.failed_attempt_count.unsigned_abs();
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
                        Expr::value(attempts.count() as i64),
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
                    failed_attempt_count: Set(attempts.count() as i64),
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

    async fn clear_failed_attempts(&self, normalized_identifier: &str) -> Result<(), RepositoryError> {
        login_attempts::Entity::delete_many()
            .filter(login_attempts::Column::NormalizedIdentifier.eq(normalized_identifier))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|_| RepositoryError::OperationFailed)
    }
}
