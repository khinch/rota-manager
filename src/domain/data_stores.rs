use super::{
    Email, LoginAttemptId, Password, ProjectId, ProjectName, TwoFACode, User,
    UserId,
};
use color_eyre::eyre::{Report, Result};
use secrecy::Secret;
use thiserror::Error;

#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError>;
    async fn delete_user(
        &mut self,
        email: &Email,
    ) -> Result<(), UserStoreError>;
}

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: &Secret<String>) -> Result<()>;
    async fn check_token(
        &self,
        token: &Secret<String>,
    ) -> Result<(), BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Token is banned")]
    BannedToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for BannedTokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::BannedToken, Self::BannedToken)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(
        &mut self,
        email: &Email,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait ProjectStore {
    async fn get_project_list(
        &mut self,
        user_id: &UserId,
    ) -> Result<Vec<(ProjectId, ProjectName)>>;
    async fn add_project(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
        project_name: &ProjectName,
    ) -> Result<()>;
    async fn delete_projects(&mut self, user_id: &UserId) -> Result<()>;
}

#[derive(Debug, Error)]
pub enum ProjectStoreError {
    #[error("Project ID exists")]
    ProjectIDExists,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for ProjectStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::ProjectIDExists, Self::ProjectIDExists)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}
