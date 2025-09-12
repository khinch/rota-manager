use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, UserStoreError},
};

#[tracing::instrument(name = "Delete user route handler", skip_all)]
pub async fn delete_user(
    State(state): State<AppState>,
    Json(request): Json<DeleteUserRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(Secret::new(request.email))
        .map_err(|_| AuthAPIError::ValidationError)?;

    {
        let mut user_store = state.user_store.write().await;
        user_store.delete_user(&email).await.map_err(|e| match e {
            UserStoreError::UserNotFound => AuthAPIError::UserNotFound,
            err => AuthAPIError::UnexpectedError(eyre!(err)),
        })?;
    }

    let message = format!("User deleted: {}", email.as_ref().expose_secret());
    let response = Json(DeleteUserResponse { message: message });

    Ok((StatusCode::OK, response))
}

#[derive(Deserialize)]
pub struct DeleteUserRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DeleteUserResponse {
    pub message: String,
}
