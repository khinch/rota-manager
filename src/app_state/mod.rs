use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{
    BannedTokenStore, EmailClient, ProjectStore, TwoFACodeStore, UserStore,
};
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;
pub type EmailClientType = Arc<dyn EmailClient + Send + Sync>;
pub type ProjectStoreType = Arc<RwLock<dyn ProjectStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
    pub project_store: ProjectStoreType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
        project_store: ProjectStoreType,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
            project_store,
        }
    }
}
