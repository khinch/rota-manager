use super::{Email, Password, UserId};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
    pub id: UserId,
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
            id: UserId::default(),
        }
    }
}
