use super::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberName(String);

impl MemberName {
    pub fn parse(name: String) -> Result<Self, ValidationError> {
        match name.chars().count() {
            x if x < 1 => Err(ValidationError::new(
                "Member name cannot be empty".to_string(),
            )),
            x if x > 255 => Err(ValidationError::new(
                "Max name length is 255 characters".to_string(),
            )),
            _ => Ok(Self(name.to_owned())),
        }
    }
}

impl AsRef<String> for MemberName {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[test]
fn test_valid_member_names() {
    let valid_names = ["a".to_string(), "a".repeat(255)];
    for valid_name in valid_names.iter() {
        let parsed = MemberName::parse(valid_name.to_owned())
            .expect("Failed to parse valid Member name");

        assert_eq!(parsed.as_ref(), valid_name);
    }
}

#[test]
fn test_short_member_names() {
    let short_name = "".to_string();
    let result = MemberName::parse(short_name);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().as_ref(), "Member name cannot be empty");
}

#[test]
fn test_long_member_names() {
    let long_name = "a".repeat(256);
    let result = MemberName::parse(long_name);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().as_ref(),
        "Max name length is 255 characters"
    );
}
