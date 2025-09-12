use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{Email, LoginAttemptId, TwoFACode},
    utils::auth::generate_auth_cookie,
    AuthAPIError,
};

#[tracing::instrument(name = "Verify 2FA route handler", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(Secret::new(request.email)) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::ValidationError)),
    };

    let login_attempt_id =
        match LoginAttemptId::parse(Secret::new(request.login_attempt_id)) {
            Ok(login_attempt_id) => login_attempt_id,
            Err(_) => return (jar, Err(AuthAPIError::ValidationError)),
        };

    let two_fa_code = match TwoFACode::parse(Secret::new(request.two_fa_code)) {
        Ok(two_fa_code) => two_fa_code,
        Err(_) => return (jar, Err(AuthAPIError::ValidationError)),
    };

    let (expected_login_attempt_id, expected_two_fa_code) =
        match state.two_fa_code_store.read().await.get_code(&email).await {
            Ok(code_tuple) => code_tuple,
            Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
        };

    if login_attempt_id != expected_login_attempt_id
        || two_fa_code != expected_two_fa_code
    {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(err) => {
            return (jar, Err(AuthAPIError::UnexpectedError(eyre!(err))))
        }
    };

    match state
        .two_fa_code_store
        .write()
        .await
        .remove_code(&email)
        .await
    {
        Ok(()) => (),
        Err(err) => {
            return (jar, Err(AuthAPIError::UnexpectedError(eyre!(err))))
        }
    };

    let updated_jar = jar.add(auth_cookie);
    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    email: String,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    two_fa_code: String,
}
