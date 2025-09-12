use std::collections::HashMap;

use crate::domain::{
    Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(
        &mut self,
        email: &Email,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some((id, code)) => Ok((id.clone(), code.clone())),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::Secret;

    fn get_test_data() -> (Email, LoginAttemptId, TwoFACode) {
        let email = Email::parse(Secret::new(String::from("foo@bar.com")))
            .expect("Could not parse email");
        let id = LoginAttemptId::parse(Secret::new(String::from(
            "b65b6b5a-cae7-436b-8196-16abcfb59e47",
        )))
        .expect("Could not parse LoginAttemptId");
        let code = TwoFACode::parse(Secret::new(String::from("123456")))
            .expect("Could not parse 2FA code");
        (email, id, code)
    }

    #[tokio::test]
    async fn add_code() {
        let (email, id, code) = get_test_data();
        let mut store = HashmapTwoFACodeStore::default();
        assert_eq!(
            store
                .add_code(email.clone(), id.clone(), code.clone())
                .await,
            Ok(()),
            "Failed to add 2FA data to store"
        );
        assert!(store.codes.contains_key(&email));
    }

    #[tokio::test]
    async fn get_code() {
        let (email, id, code) = get_test_data();
        let mut store = HashmapTwoFACodeStore::default();
        assert_eq!(
            store
                .add_code(email.clone(), id.clone(), code.clone())
                .await,
            Ok(()),
            "Failed to add 2FA data to store"
        );
        assert_eq!(
            store.get_code(&email).await.unwrap(),
            (id, code),
            "Retrieved code does not match stored code"
        );
    }

    #[tokio::test]
    async fn remove_code() {
        let (email, id, code) = get_test_data();
        let mut store = HashmapTwoFACodeStore::default();
        assert_eq!(
            store
                .add_code(email.clone(), id.clone(), code.clone())
                .await,
            Ok(()),
            "Failed to add 2FA data to store"
        );
        assert!(store.codes.contains_key(&email));
        assert_eq!(
            store.remove_code(&email).await,
            Ok(()),
            "Failed to remove code"
        );
        assert!(!store.codes.contains_key(&email));
    }

    #[tokio::test]
    async fn get_non_existent_code_returns_error() {
        let (email, _id, _code) = get_test_data();
        let store = HashmapTwoFACodeStore::default();
        assert_eq!(
            store.get_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
            "Non-existent code should return error"
        );
    }

    #[tokio::test]
    async fn code_can_be_updated() {
        let (email, id, code) = get_test_data();
        let mut store = HashmapTwoFACodeStore::default();
        assert_eq!(
            store
                .add_code(email.clone(), id.clone(), code.clone())
                .await,
            Ok(()),
            "Failed to add 2FA data to store"
        );

        let updated_id = LoginAttemptId::parse(Secret::new(String::from(
            "3a6fe309-45a9-49a6-ad44-4a5411760ae3",
        )))
        .expect("Could not parse LoginAttemptId");
        let updated_code =
            TwoFACode::parse(Secret::new(String::from("654321")))
                .expect("Could not parse 2FA code");

        assert_eq!(
            store
                .add_code(
                    email.clone(),
                    updated_id.clone(),
                    updated_code.clone()
                )
                .await,
            Ok(()),
            "Failed to update 2FA data in store"
        );

        assert_eq!(
            store.get_code(&email).await.unwrap(),
            (updated_id, updated_code),
            "Retrieved code does not match updated code"
        );
    }

    #[tokio::test]
    async fn removing_code_multiple_times_is_idempotent() {
        let (email, id, code) = get_test_data();
        let mut store = HashmapTwoFACodeStore::default();
        assert_eq!(
            store
                .add_code(email.clone(), id.clone(), code.clone())
                .await,
            Ok(()),
            "Failed to add 2FA data to store"
        );
        assert!(store.codes.contains_key(&email));
        assert_eq!(
            store.remove_code(&email).await,
            Ok(()),
            "Failed to remove code"
        );
        assert!(!store.codes.contains_key(&email));

        assert_eq!(
            store.remove_code(&email).await,
            Ok(()),
            "Failed attempt to remove non-existent code"
        );
        assert!(!store.codes.contains_key(&email));
    }
}
