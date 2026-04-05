use color_eyre::eyre::Result;
use secrecy::{ExposeSecret, Secret};
use std::collections::HashSet;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    banned_tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: &Secret<String>) -> Result<()> {
        self.banned_tokens.insert(token.expose_secret().to_owned());
        Ok(())
    }

    async fn check_token(
        &self,
        token: &Secret<String>,
    ) -> Result<(), BannedTokenStoreError> {
        if self.banned_tokens.contains(token.expose_secret()) {
            Err(BannedTokenStoreError::BannedToken)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut banned_tokens = HashsetBannedTokenStore::default();
        let token = Secret::new("token".to_owned());

        assert!(
            banned_tokens.add_token(&token).await.is_ok(),
            "Failed to add token to store"
        );
        assert!(
            banned_tokens.add_token(&token).await.is_ok(),
            "Failed to add token to store"
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut banned_tokens = HashsetBannedTokenStore::default();
        let token = Secret::new("token".to_owned());

        assert!(
            banned_tokens.check_token(&token).await.is_ok(),
            "Token banned without existing in store"
        );
        assert!(
            banned_tokens.add_token(&token).await.is_ok(),
            "Failed to add token to store"
        );
        assert_eq!(
            banned_tokens.check_token(&token).await,
            Err(BannedTokenStoreError::BannedToken),
            "Token should be banned"
        );
    }
}
