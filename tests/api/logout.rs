use auth_service::{
    domain::BannedTokenStoreError, utils::constants::JWT_COOKIE_NAME,
};
use reqwest::Url;
use secrecy::Secret;
use test_context::test_context;

use crate::helpers::{get_random_email, TestApp};

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie(app: &mut TestApp) {
    let email = get_random_email();
    let password = "password";

    assert_eq!(
        app.post_signup(&serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": false
        }))
        .await
        .status()
        .as_u16(),
        201
    );

    let login_response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 200);

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie in jar");

    let token = Secret::new(auth_cookie.value().to_owned());

    assert_eq!(
        app.banned_token_store
            .read()
            .await
            .check_token(&token)
            .await,
        Ok(())
    );

    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Unexpected error logging out"
    );

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(auth_cookie.value().is_empty());

    assert_eq!(
        app.banned_token_store
            .read()
            .await
            .check_token(&token)
            .await,
        Err(BannedTokenStoreError::BannedToken)
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row(app: &mut TestApp) {
    let email = get_random_email();
    let password = "password";

    assert_eq!(
        app.post_signup(&serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": false
        }))
        .await
        .status()
        .as_u16(),
        201
    );

    assert_eq!(
        app.post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await
        .status()
        .as_u16(),
        200
    );

    assert_eq!(app.post_logout().await.status().as_u16(), 200);
    assert_eq!(app.post_logout().await.status().as_u16(), 400);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing(app: &mut TestApp) {
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_invalid_token(app: &mut TestApp) {
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
}
