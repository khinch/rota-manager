use crate::domain::{Email, Password, User, UserStore, UserStoreError};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = &user.email;

        if self.users.contains_key(email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(email.clone(), user);
        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(&email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(&email).await?;
        if password.eq(&user.password) {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }

    async fn delete_user(
        &mut self,
        email: &Email,
    ) -> Result<(), UserStoreError> {
        match self.users.remove(email) {
            Some(_) => Ok(()),
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::{ExposeSecret, Secret};

    fn get_test_users() -> Vec<User> {
        vec![
            User::new(
                Email::parse(Secret::new("test@example.com".to_string()))
                    .unwrap(),
                Password::parse(Secret::new("P@55w0rd".to_string())).unwrap(),
                true,
            ),
            User::new(
                Email::parse(Secret::new("foo@bar.com".to_string())).unwrap(),
                Password::parse(Secret::new("ABCD1234".to_string())).unwrap(),
                false,
            ),
        ]
    }

    #[tokio::test]
    async fn test_add_user() {
        let mut users = HashmapUserStore::default();

        for test_user in get_test_users() {
            assert_eq!(
                users.add_user(test_user.clone()).await,
                Ok(()),
                "Failed to add user: {:?}",
                &test_user
            );
            assert_eq!(
                users.add_user(test_user.clone()).await,
                Err(UserStoreError::UserAlreadyExists),
                "Should not be able to add user with duplicate email"
            );
        }
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut users = HashmapUserStore::default();

        for test_user in get_test_users() {
            users.add_user(test_user.clone()).await.unwrap();

            assert_eq!(
                users.get_user(&test_user.email).await,
                Ok(test_user.clone()),
                "Failed to get user with email: {:?}",
                &test_user.email
            );
        }

        let non_existent_user =
            Email::parse(Secret::new("no@email.com".to_string())).unwrap();
        assert_eq!(
            users.get_user(&non_existent_user).await,
            Err(UserStoreError::UserNotFound),
            "User should not exist"
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut users = HashmapUserStore::default();
        let existent_email =
            Email::parse(Secret::new("foo@bar.com".to_string())).unwrap();
        let non_existent_email =
            Email::parse(Secret::new("lorem@ipsum.com".to_string())).unwrap();
        let existent_password =
            Password::parse(Secret::new("P@55w0rd".to_string())).unwrap();
        let non_existent_password =
            Password::parse(Secret::new("P155w0rd".to_string())).unwrap();

        users
            .add_user(User::new(
                existent_email.clone(),
                existent_password.clone(),
                true,
            ))
            .await
            .unwrap();

        assert_eq!(
            users
                .validate_user(&existent_email, &existent_password)
                .await,
            Ok(()),
            "User should exist with a valid password"
        );

        assert_eq!(
            users
                .validate_user(&non_existent_email, &existent_password)
                .await,
            Err(UserStoreError::UserNotFound),
            "User should not exist"
        );

        assert_eq!(
            users
                .validate_user(&existent_email, &non_existent_password)
                .await,
            Err(UserStoreError::InvalidCredentials),
            "User credentials should be invalid"
        );
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut users = HashmapUserStore::default();

        let user = User::new(
            Email::parse(Secret::new("test@example.com".to_string())).unwrap(),
            Password::parse(Secret::new("P@55w0rd".to_string())).unwrap(),
            true,
        );

        // Should be able to re-add and re-delete
        for _ in 0..2 {
            users
                .add_user(user.clone())
                .await
                .expect(user.email.as_ref().expose_secret());

            assert_eq!(
                users.delete_user(&user.email).await,
                Ok(()),
                "Failed to delete user"
            );
            assert_eq!(
                users.delete_user(&user.email).await,
                Err(UserStoreError::UserNotFound),
                "User should not have existed"
            );
        }
    }
}
