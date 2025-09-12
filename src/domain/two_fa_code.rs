use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug)]
pub struct TwoFACode(Secret<String>);

impl TwoFACode {
    pub fn parse(code: Secret<String>) -> Result<Self> {
        let regex = regex::Regex::new(r"^\d{6}$")
            .expect("Regex for TwoFACode parser is invalid");
        if regex.is_match(code.expose_secret()) {
            Ok(Self(code))
        } else {
            Err(eyre!("Code is invalid"))
        }
    }
}

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let code = rand::random::<u32>() % 1_000_000;
        TwoFACode(Secret::new(format!("{:06}", code)))
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_codes() {
        let valid_codes = ["123456", "654321", "000000", "999999"];
        for valid_code in valid_codes.iter() {
            let parsed = TwoFACode::parse(Secret::new(valid_code.to_string()))
                .expect(valid_code);
            assert_eq!(
                &parsed.as_ref().expose_secret(),
                valid_code,
                "Code does not match expected value"
            );
        }
    }

    #[test]
    fn test_invalid_codes() {
        let invalid_codes = ["12345", "1234567", "12345a", "a12345"];
        for invalid_code in invalid_codes.iter() {
            let result =
                TwoFACode::parse(Secret::new(invalid_code.to_string()));
            let error = result.expect_err(invalid_code);
            assert_eq!(error.to_string(), "Code is invalid");
        }
    }
}
