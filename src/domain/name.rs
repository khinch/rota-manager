#[macro_export]
macro_rules! name {
    ($name:ident) => {
        #[derive(
            Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
        )]
        pub struct $name(String);

        impl $name {
            pub fn parse(
                name: &str,
            ) -> Result<Self, crate::domain::ValidationError> {
                match name.chars().count() {
                    x if x < 1 => Err(crate::domain::ValidationError::new(
                        "Name cannot be empty".to_string(),
                    )),
                    x if x > 255 => Err(crate::domain::ValidationError::new(
                        "Max name length is 255 characters".to_string(),
                    )),
                    _ => Ok(Self(name.to_owned())),
                }
            }

            pub fn new(name: &str) -> Self {
                Self(name.to_string())
            }
        }

        impl AsRef<String> for $name {
            fn as_ref(&self) -> &String {
                &self.0
            }
        }
    };
}

#[cfg(test)]
mod tests {
    name!(TestName);

    #[test]
    fn test_valid_names() {
        let valid_names = ["a".to_string(), "a".repeat(255)];
        for valid_name in valid_names.iter() {
            let parsed = TestName::parse(valid_name)
                .expect("Failed to parse valid name");

            assert_eq!(parsed.as_ref(), valid_name);
        }
    }

    #[test]
    fn test_short_names() {
        let short_name = "";
        let result = TestName::parse(short_name);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().as_ref(), "Name cannot be empty");
    }

    #[test]
    fn test_long_names() {
        let long_name = "a".repeat(256);
        let result = TestName::parse(&long_name);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().as_ref(),
            "Max name length is 255 characters"
        );
    }

    #[test]
    fn invoke_new() {
        let name_str = "new name";
        let name = TestName::new(name_str);
        assert_eq!(name.as_ref(), name_str);
    }
}
