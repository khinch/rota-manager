use super::Password;
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash,
    PasswordHasher, PasswordVerifier, Version,
};
use color_eyre::eyre::{Result, WrapErr};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct UserPasswordHash(Secret<String>);

impl PartialEq for UserPasswordHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl UserPasswordHash {
    pub fn parse(s: Secret<String>) -> Result<Self> {
        let _hash = PasswordHash::new(s.expose_secret())
            .wrap_err("Failed to parse password hash")?;
        Ok(Self(s))
    }

    pub async fn from_password(password: Password) -> Result<Self> {
        let hash = Self::compute(password.as_ref().to_owned()).await?;
        Ok(hash)
    }

    async fn compute(s: Secret<String>) -> Result<Self> {
        let hash = compute_password_hash(s).await?;
        Ok(Self(hash))
    }
}

impl AsRef<Secret<String>> for UserPasswordHash {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
pub async fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();

    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash.expose_secret())?;

            Argon2::default()
                .verify_password(
                    password_candidate.expose_secret().as_bytes(),
                    &expected_password_hash,
                )
                .wrap_err("failed to verify password hash")
        })
    })
    .await?
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
pub async fn compute_password_hash(
    password: Secret<String>,
) -> Result<Secret<String>> {
    let current_span: tracing::Span = tracing::Span::current();

    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString =
                SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)?
            .to_string();

            Ok(Secret::new(password_hash))
        })
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::Secret;

    /*
     * Ensure that our implementation is capable of hashing and retrieving any
     * input that the Password validator allows. Using increasing complexity
     * makes debugging quicker if one fails.
     */
    const VALID_PASSWORDS: [&'static str; 3] = [
        "passw123",
        r#"Ab1:\n☀😎"#,
        r##"Ab1:\n☀😎`¬!"£$%^&*()_-=+[]{}|\'@#~;:/?<>,.\\\\\\\\\\☀☁☃☄★☆☎☏☐☑☒☕"##,
    ];

    #[tokio::test]
    async fn hash_valid_passwords() {
        for password in VALID_PASSWORDS.iter() {
            let secret = Secret::new(password.to_string());
            let result = UserPasswordHash::compute(secret).await;
            assert!(
                result.is_ok(),
                "Failed to hash valid password: {}",
                password
            );
        }
    }

    #[tokio::test]
    async fn verify_valid_passwords() {
        for password in VALID_PASSWORDS.iter() {
            let candidate = Secret::new(password.to_string());
            let expected = UserPasswordHash::compute(candidate.clone())
                .await
                .expect("Failed to hash valid password");
            let result =
                verify_password_hash(expected.as_ref().to_owned(), candidate)
                    .await;

            assert!(
                result.is_ok(),
                "Failed to verify valid hash for password: {}",
                password
            );
        }
    }

    #[tokio::test]
    async fn from_valid_passwords() {
        for password in VALID_PASSWORDS.iter() {
            let candidate = Secret::new(password.to_string());

            let parsed_password = Password::parse(candidate.clone())
                .expect("Failed to parse valid password");

            let expected = UserPasswordHash::from_password(parsed_password)
                .await
                .expect("Failed to hash valid password");

            let result =
                verify_password_hash(expected.as_ref().to_owned(), candidate)
                    .await;

            assert!(
                result.is_ok(),
                "Failed to verify valid hash for password: {}",
                password
            );
        }
    }
}
