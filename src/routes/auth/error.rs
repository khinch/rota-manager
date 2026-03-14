use color_eyre::eyre::Report;
use thiserror::Error;

use crate::domain::ValidationError;

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("Invalid credentials")]
    IncorrectCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Missing token")]
    MissingToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),
}
