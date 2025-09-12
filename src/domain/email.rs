use color_eyre::eyre::{Result, WrapErr};
use secrecy::{ExposeSecret, Secret};
use validator::ValidationError;

use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl Email {
    pub fn parse(s: Secret<String>) -> Result<Self> {
        if !validator::validate_email(s.expose_secret()) {
            let mut error = ValidationError::new("Invalid email address");
            error.message = Some("For more details, see the spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address".into());
            return Err(error).wrap_err("failed to parse email");
        }

        Ok(Self(s))
    }
}

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    /*
     * Validation is performed by the `validator` library.
     * Unit tests don't need to be comprehensive, just a sanity check
     * to ensure the library has been correctly implemented.
     * Docs:
     * https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address
     */
    #[test]
    fn test_valid_emails() {
        let valid_emails = ["a@b", "foo@bar.com"];
        for valid_email in valid_emails.iter() {
            let secret_email = Secret::new(valid_email.to_string());
            let parsed = Email::parse(secret_email).expect(valid_email);
            assert_eq!(
                parsed.as_ref().expose_secret().to_string(),
                valid_email.to_string(),
                "Email does not match expected value"
            );
        }
    }

    #[test]
    fn test_invalid_emails() {
        let invalid_emails = ["", "@bar.com", "ab.com", "foo.bar"];
        for invalid_email in invalid_emails.iter() {
            let secret_email = Secret::new(invalid_email.to_string());
            let result = Email::parse(secret_email);
            let error = result.expect_err(invalid_email);

            // Downcast to get the original ValidationError
            let validation_error = error
                .downcast_ref::<ValidationError>()
                .expect("Expected ValidationError");

            assert_eq!(validation_error.code, "Invalid email address");
            assert_eq!(validation_error.message.as_ref().unwrap(), "For more details, see the spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address");
        }
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(
        valid_email: ValidEmailFixture,
    ) -> bool {
        Email::parse(Secret::new(valid_email.0)).is_ok() // Updated!
    }
}
