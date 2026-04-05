use color_eyre::eyre::Report;
use thiserror::Error;

use crate::{
    application::projects::ApplicationError, domain::ValidationError,
    routes::auth::AuthAPIError,
};

#[derive(Debug, Error)]
pub enum ProjectAPIError {
    #[error("Authentication error")]
    AuthenticationError(#[from] AuthAPIError),
    #[error("{id_type} not found: {id}")]
    IDNotFoundError { id_type: String, id: uuid::Uuid },
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),
}

impl From<ApplicationError> for ProjectAPIError {
    fn from(err: ApplicationError) -> Self {
        use ApplicationError::*;

        match err {
            MemberIDNotFound(id) => ProjectAPIError::IDNotFoundError {
                id_type: "MemberID".to_string(),
                id: id.as_ref().to_owned(),
            },
            ProjectIDNotFound(id) => ProjectAPIError::IDNotFoundError {
                id_type: "ProjectID".to_string(),
                id: id.as_ref().to_owned(),
            },
            UnexpectedError(report) => ProjectAPIError::UnexpectedError(report),
            ShiftValidationError(message) => {
                ProjectAPIError::ValidationError(message)
            }
        }
    }
}
