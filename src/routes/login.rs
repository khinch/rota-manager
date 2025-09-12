use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{
        AuthAPIError, Email, LoginAttemptId, Password, TwoFACode,
        UserStoreError,
    },
    utils::auth::generate_auth_cookie,
};

#[tracing::instrument(name = "Login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let email = match Email::parse(Secret::new(request.email)) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::ValidationError)),
    };

    let password = match Password::parse(request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::ValidationError)),
    };

    let user_store = &state.user_store.read().await;

    match user_store.validate_user(&email, &password).await {
        Ok(()) => (),
        Err(UserStoreError::InvalidCredentials) => {
            return (jar, Err(AuthAPIError::IncorrectCredentials))
        }
        Err(UserStoreError::UserNotFound) => {
            return (jar, Err(AuthAPIError::IncorrectCredentials))
        }
        Err(err) => {
            return (jar, Err(AuthAPIError::UnexpectedError(err.into())))
        }
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
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
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
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
        Err(err) => {
            return (jar, Err(AuthAPIError::UnexpectedError(err.into())))
        }
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
        Err(err) => return (jar, Err(AuthAPIError::UnexpectedError(err))),
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: String::from("2FA required"),
        login_attempt_id: String::from(
            login_attempt_id.as_ref().expose_secret(),
        ),
    }));

    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

#[tracing::instrument(name = "Handling login without 2FA", skip_all)]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(err) => {
            return (jar, Err(AuthAPIError::UnexpectedError(err.into())))
        }
    };

    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
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
