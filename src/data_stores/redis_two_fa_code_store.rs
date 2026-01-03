use std::{sync::Arc, u64};

use color_eyre::eyre::{eyre, WrapErr};
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::sync::RwLock;

use crate::domain::{
    Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    #[tracing::instrument(
        name = "Adding code to Redis 2FA code store",
        skip_all
    )]
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);

        let two_fa_details = TwoFATuple(
            login_attempt_id.as_ref().expose_secret().to_owned(),
            code.as_ref().expose_secret().to_owned(),
        );

        let two_fa_details = serde_json::to_string(&two_fa_details)
            .wrap_err("failed to serialise 2FA tuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex::<_, _, ()>(key, two_fa_details, TEN_MINUTES_IN_SECONDS)
            .wrap_err("failed to set 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(
        name = "removing code from Redis 2FA code store",
        skip_all
    )]
    async fn remove_code(
        &mut self,
        email: &Email,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);

        self.conn
            .write()
            .await
            .del::<_, ()>(key)
            .wrap_err("failed to delete 2FA code from Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(
        name = "getting code from Redis 2FA code store",
        skip_all
    )]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(&email);

        let two_fa_details =
            self.conn.write().await.get::<_, String>(key).map_err(
                |e| match e.kind() {
                    redis::ErrorKind::TypeError => {
                        TwoFACodeStoreError::LoginAttemptIdNotFound
                    }
                    redis::ErrorKind::IoError => {
                        TwoFACodeStoreError::UnexpectedError(eyre!(e))
                    }
                    redis::ErrorKind::ResponseError => {
                        TwoFACodeStoreError::UnexpectedError(eyre!(e))
                    }
                    _ => TwoFACodeStoreError::UnexpectedError(eyre!(e)),
                },
            )?;

        let two_fa_details =
            serde_json::from_str::<TwoFATuple>(&two_fa_details)
                .wrap_err("failed to deserialise 2FA tuple")
                .map_err(TwoFACodeStoreError::UnexpectedError)?;

        let (login_attempt_id, code) = (
            LoginAttemptId::parse(Secret::new(two_fa_details.0))
                .map_err(|e| TwoFACodeStoreError::UnexpectedError(eyre!(e)))?,
            TwoFACode::parse(Secret::new(two_fa_details.1))
                .map_err(|e| TwoFACodeStoreError::UnexpectedError(eyre!(e)))?,
        );

        Ok((login_attempt_id, code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

#[tracing::instrument(name = "building key for Redis 2FA code store", skip_all)]
fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref().expose_secret())
}
