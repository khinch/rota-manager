use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie, CookieJar};
use color_eyre::eyre::eyre;
use secrecy::Secret;

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    AppState,
};

#[tracing::instrument(name = "Logout route handler", skip_all)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = Secret::new(cookie.value().to_string());

    match validate_token(&token, state.banned_token_store.clone()).await {
        Ok(_) => (),
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
    };

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

    (jar, Ok(StatusCode::OK))
}
