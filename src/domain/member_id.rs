use super::ValidationError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberId(Uuid);

impl MemberId {
    pub fn parse(id: &str) -> Result<Self, ValidationError> {
        let parsed = uuid::Uuid::try_parse(id).map_err(|e| {
            ValidationError::new(format!("Invalid member ID: {e}"))
        })?;
        Ok(Self(parsed))
    }

    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for MemberId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl AsRef<Uuid> for MemberId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[test]
fn test_valid_ids() {
    let valid_id = "5e90ca28-e1ad-4795-a190-089959c16e0b";
    let parsed = MemberId::parse(valid_id).expect(valid_id);
    assert_eq!(
        parsed.as_ref().to_string(),
        valid_id,
        "ID does not match expected value"
    );
}

#[test]
fn test_invalid_ids() {
    let invalid_id = "5b5b32e3a66cc-45bc-82d1-d41582139f1e";
    let result = MemberId::parse(invalid_id);
    let error = result.expect_err(invalid_id);
    assert_eq!(error.as_ref(), "Invalid member ID: failed to parse a UUID");
}
