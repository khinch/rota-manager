use super::{Email, UserId, UserPasswordHash};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct User {
    pub email: Email,
    pub hash: UserPasswordHash,
    pub requires_2fa: bool,
    pub id: UserId,
}

impl User {
    pub fn new(
        email: Email,
        hash: UserPasswordHash,
        requires_2fa: bool,
    ) -> Self {
        Self {
            email,
            hash,
            requires_2fa,
            id: UserId::default(),
        }
    }
}
