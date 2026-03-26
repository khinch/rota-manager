use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
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
