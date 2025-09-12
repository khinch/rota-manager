use color_eyre::eyre::{eyre, Result, WrapErr};
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    domain::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(
        name = "Adding token to Redis banned token store",
        skip_all
    )]
    async fn add_token(&mut self, token: &Secret<String>) -> Result<()> {
        let key = get_key(token);
        let token_ttl_seconds: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex::<_, _, ()>(key, true, token_ttl_seconds)
            .wrap_err("failed to set banned token in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Checking Redis banned token store", skip_all)]
    async fn check_token(
        &self,
        token: &Secret<String>,
    ) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token);
        match self.conn.write().await.exists(&key) {
            Ok(true) => Err(BannedTokenStoreError::BannedToken),
            Ok(false) => Ok(()),
            Err(e) => {
                Err(eyre!(e)
                    .wrap_err("failed to check if token exists in Redis"))
                .map_err(BannedTokenStoreError::UnexpectedError)
            }
        }
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &Secret<String>) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token.expose_secret())
}
