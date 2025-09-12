use crate::helpers::{get_random_email, TestApp};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_token(app: &mut TestApp) {
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
        .expect("No auth cookie found");

    let token = auth_cookie.value();

    assert_eq!(
        app.post_verify_token(&serde_json::json!(
            {
                "token": token
            }
        ))
        .await
        .status()
        .as_u16(),
        200
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_invalid_token(app: &mut TestApp) {
    let token = format!(
        "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
        JWT_COOKIE_NAME
    );
    let body = serde_json::json!({ "token": token });

    let response = app.post_verify_token(&body).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_input(app: &mut TestApp) {
    let invalid_requests = [
        json!({"token": true}),
        json!({"token": 1}),
        json!({"broken": "token"}),
    ];

    for request in invalid_requests {
        let response = app.post_verify_token(&request).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_banned_token(app: &mut TestApp) {
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

    let token = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found")
        .value()
        .to_owned();

    let verify_token_body = serde_json::json!({"token": &token});

    assert_eq!(
        app.post_verify_token(&verify_token_body)
            .await
            .status()
            .as_u16(),
        200
    );

    assert_eq!(app.post_logout().await.status().as_u16(), 200);

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 401);
}
