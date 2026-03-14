use color_eyre::eyre::Report;
use thiserror::Error;

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
