#[macro_export]
macro_rules! id {
    ($name:ident) => {
        // use crate::domain::error::ValidationError;

        #[derive(
            Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize,
        )]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn parse(
                id: &str,
            ) -> Result<Self, crate::domain::error::ValidationError> {
                let parsed = uuid::Uuid::try_parse(id).map_err(|e| {
                    crate::domain::error::ValidationError::new(format!(
                        "Invalid ID: {e}"
                    ))
                })?;
                Ok(Self(parsed))
            }

            pub fn new(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl AsRef<uuid::Uuid> for $name {
            fn as_ref(&self) -> &uuid::Uuid {
                &self.0
            }
        }
    };
}

#[cfg(test)]
mod tests {
    id!(TestId);

    #[test]
    fn test_parse_valid_id() {
        let valid_id = "5e90ca28-e1ad-4795-a190-089959c16e0b";
        let parsed = TestId::parse(valid_id).expect(valid_id);
        assert_eq!(
            parsed.as_ref().to_string(),
            valid_id,
            "ID does not match expected value"
        );
    }

    #[test]
    fn test_parse_invalid_id() {
        let invalid_id = "5b5b32e3a66cc-45bc-82d1-d41582139f1e";
        let result = TestId::parse(invalid_id);
        let error = result.expect_err(invalid_id);
        assert_eq!(error.as_ref(), "Invalid ID: failed to parse a UUID");
    }

    #[test]
    fn test_new_id() {
        let valid_uuid = uuid::Uuid::new_v4();
        let copied = valid_uuid.to_owned();
        let test_id = TestId::new(valid_uuid);
        assert_eq!(test_id.as_ref(), &copied);
    }

    #[test]
    fn test_default_id() {
        let default_id = TestId::default();
        assert!(uuid::Uuid::try_parse(&default_id.as_ref().to_string()).is_ok());
    }
}
