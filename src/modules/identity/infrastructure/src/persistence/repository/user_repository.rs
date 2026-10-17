use async_trait::async_trait;
use identity_application::ports::{
    CredentialChange, InitialAdminRegistration, MemberRegistration, RepositoryError, UserRepository,
};
use identity_domain::{AvatarRef, DisplayName, Email, Role, User};
use kernel::UserId;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};

use super::timestamps::to_rfc3339;
use super::user_row::{insert_model, is_unique_violation, restore_user, ROLE_ADMIN};
use super::SeaOrmUserRepository;
use crate::persistence::{login_attempts, users};

const SETUP_CLAIM_SQL: &str = "DELETE FROM instance_state WHERE singleton = 1";

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        users::Entity::find()
            .count(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        users::Entity::find()
            .order_by_asc(users::Column::Email)
            .all(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?
            .into_iter()
            .map(restore_user)
            .collect()
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

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        users::Entity::find_by_id(id.value())
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

    async fn insert_member(
        &self,
        registration: MemberRegistration,
    ) -> Result<User, RepositoryError> {
        let user = User::new_temporary(
            UserId::new(0),
            registration.email().clone(),
            registration.password_hash().clone(),
            Role::Member,
            registration.created_at(),
        )
        .map_err(|_| RepositoryError::OperationFailed)?;
        let model = insert_model(&user, registration.email().as_str());
        let inserted = model.insert(&self.db).await.map_err(|error| {
            if is_unique_violation(&error) {
                RepositoryError::EmailAlreadyExists
            } else {
                RepositoryError::OperationFailed
            }
        })?;
        restore_user(inserted)
    }

    async fn set_disabled(&self, id: UserId, disabled: bool) -> Result<(), RepositoryError> {
        let result = users::Entity::update_many()
            .col_expr(users::Column::Disabled, Expr::value(disabled))
            .filter(users::Column::Id.eq(id.value()))
            .exec(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        if result.rows_affected == 0 {
            return Err(RepositoryError::UserNotFound);
        }
        Ok(())
    }

    async fn set_profile(
        &self,
        id: UserId,
        display_name: Option<DisplayName>,
        avatar: Option<AvatarRef>,
    ) -> Result<User, RepositoryError> {
        let existing = users::Entity::find_by_id(id.value())
            .one(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?
            .ok_or(RepositoryError::UserNotFound)?;

        let mut model: users::ActiveModel = existing.into();
        model.display_name = Set(display_name.map(|name| name.as_str().to_owned()));
        model.avatar_ext = Set(avatar.as_ref().map(|avatar| avatar.extension().to_owned()));
        model.avatar_key = Set(avatar.as_ref().map(|avatar| avatar.key().to_owned()));
        model
            .update(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;

        self.find_by_id(id)
            .await?
            .ok_or(RepositoryError::UserNotFound)
    }

    async fn delete_user(&self, id: UserId) -> Result<bool, RepositoryError> {
        let result = users::Entity::delete_by_id(id.value())
            .exec(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        Ok(result.rows_affected > 0)
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        users::Entity::find()
            .filter(users::Column::Role.eq(ROLE_ADMIN))
            .filter(users::Column::Disabled.eq(false))
            .count(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)
    }

    async fn delete_login_attempts(&self, normalized_email: &str) -> Result<(), RepositoryError> {
        login_attempts::Entity::delete_many()
            .filter(login_attempts::Column::NormalizedIdentifier.eq(normalized_email))
            .exec(&self.db)
            .await
            .map_err(|_| RepositoryError::OperationFailed)?;
        Ok(())
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
        let changed = match credential_change.expires_at() {
            None => current
                .replace_with_permanent_password(credential_change.password_hash().clone())
                .map_err(|_| RepositoryError::AuthVersionOverflow)?,
            Some(expires_at) => current
                .replace_with_temporary_password(
                    credential_change.password_hash().clone(),
                    expires_at,
                )
                .map_err(|_| RepositoryError::AuthVersionOverflow)?,
        };
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
                Expr::value(
                    i64::try_from(changed.auth_version())
                        .map_err(|_| RepositoryError::OperationFailed)?,
                ),
            )
            .col_expr(
                users::Column::MustChangePassword,
                Expr::value(changed.must_change_password()),
            )
            .col_expr(
                users::Column::PasswordExpiresAt,
                Expr::value(
                    changed
                        .password_expires_at()
                        .map(|expires_at| to_rfc3339(&expires_at)),
                ),
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
