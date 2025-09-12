use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    domain::Email, routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME, ErrorResponse,
};

use secrecy::{ExposeSecret, Secret};
use test_context::test_context;
use wiremock::{matchers::method, matchers::path, Mock, ResponseTemplate};

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled(
    app: &mut TestApp,
) {
    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled(
    app: &mut TestApp,
) {
    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");
    assert_eq!(json_body.message, String::from("2FA required"));

    let email = Email::parse(Secret::new(String::from(&random_email)))
        .expect("Failed to parse email");

    let (expected_id, _two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .expect("Failed to get 2FA data from store");
    assert_eq!(
        json_body.login_attempt_id,
        String::from(expected_id.as_ref().expose_secret()),
        "2FA IDs do not match"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_credentials(app: &mut TestApp) {
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": random_email
        }),
        serde_json::json!({
            "password": "password"
        }),
        serde_json::json!({
            "email": true,
            "password": "password"
        }),
        serde_json::json!({
            "email": random_email,
            "password": true,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_invalid_input(app: &mut TestApp) {
    let test_cases = [
        serde_json::json!({
            "email": "foobar.com",
            "password": "abcd1234",
        }),
        serde_json::json!({
            "email": "",
            "password": "abcd1234",
        }),
        serde_json::json!({
            "email": "a@b.com",
            "password": "abcd123",
        }),
        serde_json::json!({
            "email": "a@b.com",
            "password": "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ123",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Should fail with HTTP400 for input: {}",
            test_case
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialise response body to ErrorResponse")
                .error,
            "Invalid input".to_owned()
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_credentials_incorrect(app: &mut TestApp) {
    let email = get_random_email();
    let password = String::from("abcd1234");

    let signup_data = serde_json::json!({
        "email": email,
        "password": password,
        "requires2FA": false
    });

    let response = app.post_signup(&signup_data).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Should return 201 for new account created with data: {}",
        signup_data
    );

    let login_data = [
        serde_json::json!({
            "email": "foo@bar.com",
            "password": password
        }),
        serde_json::json!({
            "email": email,
            "password": "incorrect"
        }),
    ];

    for invalid_login in login_data {
        let response = app.post_login(&invalid_login).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Should fail with HTTP401 (Incorrect credentials)"
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialise response body to ErrorResponse")
                .error,
            "Incorrect credentials".to_owned()
        );
    }
}
