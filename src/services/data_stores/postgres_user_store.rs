use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::domain::{
    verify_password_hash, Email, Password, User, UserId, UserPasswordHash,
    UserStore, UserStoreError,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, requires_2fa) VALUES ($1, $2, $3, $4)
            "#,
            user.id.as_ref() as &uuid::Uuid,
            user.email.as_ref().expose_secret(),
            user.hash.as_ref().expose_secret(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                UserStoreError::UserAlreadyExists
            }
            err => UserStoreError::UnexpectedError(eyre!(err)),
        })?;
        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        sqlx::query!(
            r#"
                    SELECT id, email, password_hash, requires_2fa
                    FROM users
                    WHERE email = $1
                    "#,
            email.as_ref().expose_secret()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
            err => UserStoreError::UnexpectedError(eyre!(err)),
        })
        .map(|row| {
            Ok(User {
                id: UserId::new(row.id),
                email: Email::parse(Secret::new(row.email))
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                hash: UserPasswordHash::parse(Secret::new(row.password_hash))
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                requires_2fa: row.requires_2fa,
            })
        })?
    }

    #[tracing::instrument(
        name = "Validating user credentials in PostgreSQL",
        skip_all
    )]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(&email).await?;
        verify_password_hash(
            user.hash.as_ref().to_owned(),
            password.as_ref().to_owned(),
        )
        .await
        .map_err(|_| UserStoreError::InvalidCredentials)
    }

    async fn delete_user(
        &mut self,
        email: &Email,
    ) -> Result<(), UserStoreError> {
        let result = sqlx::query!(
            r#"
               DELETE FROM users WHERE email = $1
               "#,
            email.as_ref().expose_secret()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }
}
