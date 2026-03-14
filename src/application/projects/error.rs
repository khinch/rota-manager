use color_eyre::eyre::Report;
use thiserror::Error;

use crate::domain::{MemberId, ProjectId};

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Member ID {0} not found")]
    MemberIDNotFound(MemberId),
    #[error("Project ID {0} not found")]
    ProjectIDNotFound(ProjectId),
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[cfg(test)]
impl PartialEq for ApplicationError {
    fn eq(&self, other: &Self) -> bool {
        use ApplicationError::*;

        match (self, other) {
            (MemberIDNotFound(a), MemberIDNotFound(b)) => a == b,
            (ProjectIDNotFound(a), ProjectIDNotFound(b)) => a == b,
            (UnexpectedError(_), UnexpectedError(_)) => true,
            _ => false,
        }
    }
}
