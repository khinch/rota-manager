use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{
        AuthAPIError, Email, LoginAttemptId, Password, TwoFACode, UserId,
        UserStoreError,
    },
    utils::auth::generate_auth_cookie,
};

#[tracing::instrument(name = "Login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<LoginResponse>), AuthAPIError> {
    let email = Email::parse(Secret::new(request.email))?;
    let password = Password::parse(request.password)?;
    let user_store = &state.user_store.read().await;

    user_store
        .validate_user(&email, &password)
        .await
        .map_err(|e| match e {
            UserStoreError::InvalidCredentials
            | UserStoreError::UserNotFound => {
                AuthAPIError::IncorrectCredentials
            }
            _ => AuthAPIError::UnexpectedError(eyre!(e)),
        })?;

    let user = user_store
        .get_user(&email)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, &user.id, jar).await,
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: Secret<String>,
}

#[tracing::instrument(name = "Handling 2FA login", skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<LoginResponse>), AuthAPIError> {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    match state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
    {
        Ok(()) => (),
        Err(e) => return Err(AuthAPIError::UnexpectedError(eyre!(e))),
    }

    match state
        .email_client
        .send_email(
            &email,
            "LGR Bootcamp 2FA Code",
            two_fa_code.as_ref().expose_secret(),
        )
        .await
    {
        Ok(()) => (),
        Err(e) => return Err(AuthAPIError::UnexpectedError(e)),
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: String::from("2FA required"),
        login_attempt_id: String::from(
            login_attempt_id.as_ref().expose_secret(),
        ),
    }));

    Ok((StatusCode::PARTIAL_CONTENT, jar, response))
}

#[tracing::instrument(name = "Handling login without 2FA", skip_all)]
async fn handle_no_2fa(
    email: &Email,
    user_id: &UserId,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<LoginResponse>), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(&email, &user_id)
        .map_err(|e| AuthAPIError::UnexpectedError(eyre!(e)))?;

    let updated_jar = jar.add(auth_cookie);

    Ok((
        StatusCode::OK,
        updated_jar,
        Json(LoginResponse::RegularAuth),
    ))
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
