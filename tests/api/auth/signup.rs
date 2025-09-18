use crate::helpers::{get_random_email, TestApp};
use rota_manager::{routes::auth::SignupResponse, ErrorResponse};
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_input(app: &mut TestApp) {
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "requires2FA": true
        }),
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123"
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": "true"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
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
async fn should_return_201_for_valid_requests(app: &mut TestApp) {
    let test_cases = [
        serde_json::json!({
            "email": get_random_email(),
            "password": "abcd1234",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "abcd1234",
            "requires2FA": false
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12",
            "requires2FA": false
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            201,
            "Failed for input: {:?}",
            test_case
        );

        let expected_response = SignupResponse {
            message: "User created successfully".to_owned(),
        };

        assert_eq!(
            response
                .json::<SignupResponse>()
                .await
                .expect("Could not deserialise response body to UserBody"),
            expected_response
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_invalid_input(app: &mut TestApp) {
    let test_cases = [
        (serde_json::json!({
            "email": "foobar.com",
            "password": "abcd1234",
            "requires2FA": true
        }), "Validation error: Invalid email address. For more details, see the spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address"),
        (serde_json::json!({
            "email": "",
            "password": "abcd1234",
            "requires2FA": true
        }), "Validation error: Invalid email address. For more details, see the spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address"),
        (serde_json::json!({
            "email": "a@b.com",
            "password": "abcd123",
            "requires2FA": true
        }), "Validation error: Password too short. Should be 8 to 128 characters."),
        (serde_json::json!({
            "email": "foo@bar.com",
            "password": "abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ123",
            "requires2FA": false
        }), "Validation error: Password too long. Should be 8 to 128 characters."),
    ];

    for (body, error_message) in test_cases.iter() {
        let response = app.post_signup(&body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Should fail with HTTP400 for input: {}",
            body
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialise response body to ErrorResponse")
                .error,
            error_message.to_string()
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_409_if_email_exists(app: &mut TestApp) {
    let email = get_random_email();

    let request_data = serde_json::json!({
        "email": email,
        "password": "abcd1234",
        "requires2FA": true
    });

    let response = app.post_signup(&request_data).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Should return 201 for new account created with data: {}",
        request_data
    );

    let response = app.post_signup(&request_data).await;
    assert_eq!(
        response.status().as_u16(),
        409,
        "Should fail with HTTP409 (account with email already exists)"
    );
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialise response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
