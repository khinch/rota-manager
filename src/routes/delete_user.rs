use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::{cookie, CookieJar};
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, UserStoreError},
    utils::{auth::get_claims, constants::JWT_COOKIE_NAME},
};

#[tracing::instrument(name = "Delete user route handler", skip_all)]
pub async fn delete_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<DeleteUserResponse>), AuthAPIError>,
) {
    let claims = match get_claims(&jar, &state.banned_token_store).await {
        Ok(claims) => claims,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
    };

    let user_id = claims.id;
    let email = match Email::parse(Secret::new(claims.sub)) {
        Ok(email) => email,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
    };

    {
        let mut project_store = state.project_store.write().await;
        match project_store.delete_projects(&user_id).await {
            Ok(()) => (),
            Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
        }
    }

    {
        let mut user_store = state.user_store.write().await;
        match user_store.delete_user(&email).await {
            Ok(()) => (),
            Err(UserStoreError::UserNotFound) => {
                return (jar, Err(AuthAPIError::UserNotFound))
            }
            Err(e) => {
                return (jar, Err(AuthAPIError::UnexpectedError(eyre!(e))))
            }
        }
    }

    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = Secret::new(cookie.value().to_string());

    match state
        .banned_token_store
        .write()
        .await
        .add_token(&token)
        .await
    {
        Ok(()) => (),
        Err(err) => {
            return (jar, Err(AuthAPIError::UnexpectedError(eyre!(err))))
        }
    }

    let jar = jar.remove(cookie::Cookie::from(JWT_COOKIE_NAME));

    let message = format!("User deleted: {}", email.as_ref().expose_secret());
    let response = Json(DeleteUserResponse { message: message });

    (jar, Ok((StatusCode::OK, response)))
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DeleteUserResponse {
    pub message: String,
}
