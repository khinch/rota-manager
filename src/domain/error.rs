use color_eyre::eyre::Report;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ProjectAPIError {
    #[error("Authentication error")]
    AuthenticationError(#[from] AuthAPIError),
    #[error("Resource with ID not found: {0}")]
    IDNotFoundError(uuid::Uuid),
    #[error("Resource with ID already exists: {0}")]
    IDExistsError(uuid::Uuid),
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),
}

#[derive(Debug, Error)]
#[error("Validation error: {0}")]
pub struct ValidationError(String);

impl ValidationError {
    pub fn new(message: String) -> Self {
        Self(message)
    }

    pub fn as_ref(&self) -> &String {
        &self.0
    }
}
