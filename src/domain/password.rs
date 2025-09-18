use super::ValidationError;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Password {
    pub fn parse(s: Secret<String>) -> Result<Password, ValidationError> {
        validate_password(&s)?;
        Ok(Self(s))
    }
}

fn validate_password(s: &Secret<String>) -> Result<(), ValidationError> {
    let min_characters = 8;
    let max_characters = 128;
    let char_count = s.expose_secret().chars().count();

    if char_count < min_characters {
        return Err(ValidationError::new(format!(
            "Password too short. Should be {} to {} characters.",
            min_characters, max_characters
        )));
    }

    if char_count > max_characters {
        return Err(ValidationError::new(format!(
            "Password too long. Should be {} to {} characters.",
            min_characters, max_characters
        )));
    }

    Ok(())
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::Password as FakePassword;
    use fake::Fake;
    use secrecy::Secret;

    #[test]
    fn test_valid_passwords() {
        let valid_passwords = [
            "12345678",
            "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12",
            r##"`¬!"£$%^&*()_-=+[]{}|\'@#~;:/?<>,.\\\\\\\\\\"##,
            "😀😁😂😃😄😅😆😎",
            "☀☁☂☃☄★☆☇☈☉☊☋☌☍☎☏☐☑☒☓☔☕ħĨ☘☙☚☛☜☝☞☟☠☡☢☣ĩ☥☦☧☨☩☪☫☬☭☮☯☰☱☲☳☴☵☶☷☸☹☺☻☼☽☾☿☀☁☂☃☄★☆☇☈☉☊☋☌☍☎☏☐☑☒☓☔☕ħĨ☘☙☚☛☜☝☞☟☠☡☢☣ĩ☥☦☧☨☩☪☫☬☭☮☯☰☱☲☳☴☵☶☷☸☹☺☻☼☽☾☿"
        ];
        for valid_password in valid_passwords.iter() {
            let secret_password = Secret::new(valid_password.to_string());
            let parsed = Password::parse(secret_password)
                .expect("Failed to parse valid password");

            assert_eq!(
                parsed.as_ref().expose_secret().to_string(),
                valid_password.to_string()
            );
        }
    }

    #[test]
    fn test_short_passwords() {
        let short_passwords = ["", "1234567", "😀😁😂😃😄😅😆"];
        for short_password in short_passwords.iter() {
            let secret_password = Secret::new(short_password.to_string());
            let result = Password::parse(secret_password);

            let error = result.expect_err(short_password);
            assert!(error.as_ref().starts_with("Password too short"));
        }
    }

    #[test]
    fn test_long_passwords() {
        let long_passwords = [
            "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ123",
            "☀☁☂☃☄★☆☇☈☉☊☋☌☍☎☏☐☑☒☓☔☕ħĨ☘☙☚☛☜☝☞☟☠☡☢☣ĩ☥☦☧☨☩☪☫☬☭☮☯☰☱☲☳☴☵☶☷☸☹☺☻☼☽☾☿☀☁☂☃☄★☆☇☈☉☊☋☌☍☎☏☐☑☒☓☔☕ħĨ☘☙☚☛☜☝☞☟☠☡☢☣ĩ☥☦☧☨☩☪☫☬☭☮☯☰☱☲☳☴☵☶☷☸☹☺☻☼☽☾☿♀",
        ];
        for long_password in long_passwords.iter() {
            let secret_password = Secret::new(long_password.to_string());
            let result = Password::parse(secret_password);
            let error = result.expect_err(long_password);

            assert!(error.as_ref().starts_with("Password too long"));
        }
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let password = FakePassword(8..30).fake_with_rng(g);
            Self(Secret::new(password))
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(
        valid_password: ValidPasswordFixture,
    ) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}
