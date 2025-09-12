use crate::helpers::{get_random_email, TestApp};
use auth_service::{routes::DeleteUserResponse, ErrorResponse};

use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_requests(app: &mut TestApp) {
    let email = get_random_email();
    let signup_request = serde_json::json!({
        "email": email,
        "password": "password",
        "requires2FA": true
    });
    let delete_user_request = serde_json::json!({
        "email": email
    });

    let create_user_response = app.post_signup(&signup_request).await;
    assert_eq!(
        create_user_response.status().as_u16(),
        201,
        "Failed to create user for testing"
    );

    let delete_user_response = app.delete_user(&delete_user_request).await;
    assert_eq!(
        delete_user_response.status().as_u16(),
        200,
        "Failed for input: {:?}",
        delete_user_response
    );

    let expected_response = DeleteUserResponse {
        message: format!("User deleted: {}", email),
    };

    assert_eq!(
        delete_user_response
            .json::<DeleteUserResponse>()
            .await
            .expect("Could not deserialise response body to UserBody"),
        expected_response
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_if_email_does_not_exists(app: &mut TestApp) {
    let email = get_random_email();
    let signup_request = serde_json::json!({
        "email": email,
        "password": "password",
        "requires2FA": true
    });
    let delete_user_request = serde_json::json!({
        "email": email
    });

    let create_user_response = app.post_signup(&signup_request).await;
    assert_eq!(
        create_user_response.status().as_u16(),
        201,
        "Failed to create user for testing"
    );

    let delete_user_response = app.delete_user(&delete_user_request).await;
    assert_eq!(
        delete_user_response.status().as_u16(),
        200,
        "Failed for input: {:?}",
        delete_user_response
    );

    let delete_user_response = app.delete_user(&delete_user_request).await;
    assert_eq!(
        delete_user_response.status().as_u16(),
        404,
        "Failed for input: {:?}",
        delete_user_response
    );
    assert_eq!(
        delete_user_response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialise response body to ErrorResponse")
            .error,
        "User not found".to_owned()
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_input(app: &mut TestApp) {
    let test_cases = [
        serde_json::json!({
            "email": true
        }),
        serde_json::json!({
            "foo": "bar"
        }),
        serde_json::json!({
            "email": 3
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.delete_user(test_case).await;
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
    let delete_user_request = serde_json::json!({
        "email": "foobar.com"
    });

    let response = app.delete_user(&delete_user_request).await;
    assert_eq!(
        response.status().as_u16(),
        400,
        "Should fail with HTTP400 for input: {}",
        delete_user_request
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
