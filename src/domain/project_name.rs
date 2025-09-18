use super::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn parse(name: &str) -> Result<Self, ValidationError> {
        match name.chars().count() {
            x if x < 1 => Err(ValidationError::new(
                "Project name cannot be empty".to_string(),
            )),
            x if x > 255 => Err(ValidationError::new(
                "Max name length is 255 characters".to_string(),
            )),
            _ => Ok(Self(name.to_owned())),
        }
    }
}

impl AsRef<String> for ProjectName {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[test]
fn test_valid_project_names() {
    let valid_names = ["a".to_string(), "a".repeat(255)];
    for valid_name in valid_names.iter() {
        let parsed = ProjectName::parse(valid_name)
            .expect("Failed to parse valid project name");

        assert_eq!(parsed.as_ref(), valid_name);
    }
}

#[test]
fn test_short_project_names() {
    let short_password = "";
    let result = ProjectName::parse(short_password);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().as_ref(), "Project name cannot be empty");
}

#[test]
fn test_long_project_names() {
    let long_password = "a".repeat(256);
    let result = ProjectName::parse(&long_password);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().as_ref(),
        "Max name length is 255 characters"
    );
}
