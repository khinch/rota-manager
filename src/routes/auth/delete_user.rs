use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::{cookie, CookieJar};
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email},
    utils::{auth::get_claims, constants::JWT_COOKIE_NAME},
};

#[tracing::instrument(name = "Delete user route handler", skip_all)]
pub async fn delete_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<DeleteUserResponse>), AuthAPIError> {
    let claims = get_claims(&jar, &state.banned_token_store).await?;

    let user_id = claims.id;

    let email = Email::parse(Secret::new(claims.sub))
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    state
        .project_store
        .write()
        .await
        .delete_projects(&user_id)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    let user_projects = state
        .project_store
        .write()
        .await
        .get_project_list(&user_id)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    {
        let mut project_store = state.project_store.write().await;
        for (project_id, _project_name) in &user_projects {
            project_store
                .delete_members(&user_id, &project_id)
                .await
                .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;
        }
    }

    state
        .project_store
        .write()
        .await
        .delete_projects(&user_id)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    state
        .user_store
        .write()
        .await
        .delete_user(&email)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    let cookie =
        jar.get(JWT_COOKIE_NAME)
            .ok_or(AuthAPIError::UnexpectedError(eyre!(
                "No JWT cookie found during delete user"
            )))?;

    let token = Secret::new(cookie.value().to_string());

    state
        .banned_token_store
        .write()
        .await
        .add_token(&token)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e))?;

    let jar = jar.remove(cookie::Cookie::from(JWT_COOKIE_NAME));

    let message = format!("User deleted: {}", email.as_ref().expose_secret());
    let response = Json(DeleteUserResponse { message: message });

    Ok((StatusCode::OK, jar, response))
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DeleteUserResponse {
    pub message: String,
}
