use crate::helpers::{get_random_email, TestApp};
use auth_service::{domain::Email, utils::constants::JWT_COOKIE_NAME};
use secrecy::{ExposeSecret, Secret};
use test_context::test_context;
use wiremock::{matchers::method, matchers::path, Mock, ResponseTemplate};

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_input(app: &mut TestApp) {
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
          "email": random_email,
          "loginAttemptId": "aeeddfff-94c3-447f-8dd8-1f779c6412c9"
        }),
        serde_json::json!({
          "email": random_email,
          "2FACode": "123456"
        }),
        serde_json::json!({
          "loginAttemptId": "aeeddfff-94c3-447f-8dd8-1f779c6412c9",
          "2FACode": "string"
        }),
        serde_json::json!({
          "email": random_email,
          "loginAttemptId": true,
          "2FACode": "123456"
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
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
    let random_email = get_random_email();
    let uuid = "aeeddfff-94c3-447f-8dd8-1f779c6412c9";
    let two_fa_code = "123456";

    let test_cases = [
        serde_json::json!({
          "email": "foobar.com",
          "loginAttemptId": uuid,
          "2FACode": two_fa_code
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "geeddfff-94c3-447f-8dd8-1f779c6412c9",
            "2FACode": two_fa_code
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": uuid,
            "2FACode": "12345"
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(app: &mut TestApp) {
    let email = get_random_email();
    let password = "password";

    let signup_request = serde_json::json!({
        "email": email,
        "password": password,
        "requires2FA": true
    });
    assert_eq!(
        app.post_signup(&signup_request).await.status().as_u16(),
        201
    );

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);

    let parsed_email = Email::parse(Secret::new(email.clone())).unwrap();

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&parsed_email)
        .await
        .unwrap();

    // Each request should be able to be parsed, but the codes should not
    // be valid
    let invalid_2fa_requests = [
        serde_json::json!({
            "email": "foo@bar.com",
            "loginAttemptId": login_attempt_id.as_ref().expose_secret(),
            "2FACode": two_fa_code.as_ref().expose_secret()
        }),
        serde_json::json!({
            "email": email,
            "loginAttemptId": "32bdc600-115d-4062-8649-8c558c00eb86",
            "2FACode": two_fa_code.as_ref().expose_secret()
        }),
        serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref().expose_secret(),
            "2FACode": "123456"
        }),
    ];

    for invalid_2fa_request in invalid_2fa_requests {
        let response = app.post_verify_2fa(&invalid_2fa_request).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Should not be able to verify invalid request: {}",
            &invalid_2fa_request
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_old_code(app: &mut TestApp) {
    let email = get_random_email();
    let parsed_email = Email::parse(Secret::new(email.clone())).unwrap();
    let password = "password";

    assert_eq!(
        app.post_signup(&serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await
        .status()
        .as_u16(),
        201
    );

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);

    let (_first_login_attempt_id, first_two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&parsed_email)
        .await
        .unwrap();

    let login_response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);

    let (second_login_attempt_id, _second_two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&parsed_email)
        .await
        .unwrap();

    let outdated_two_fa_request = serde_json::json!({
      "email": email,
      "loginAttemptId": second_login_attempt_id.as_ref().expose_secret(),
      "2FACode": first_two_fa_code.as_ref().expose_secret()
    });

    let response = app.post_verify_2fa(&outdated_two_fa_request).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Should not be able to verify old code"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_if_correct_code(app: &mut TestApp) {
    let email = get_random_email();
    let parsed_email = Email::parse(Secret::new(email.clone())).unwrap();
    let password = "password";

    assert_eq!(
        app.post_signup(&serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await
        .status()
        .as_u16(),
        201
    );

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&parsed_email)
        .await
        .unwrap();

    let two_fa_request = serde_json::json!({
      "email": email,
      "loginAttemptId": login_attempt_id.as_ref().expose_secret(),
      "2FACode": two_fa_code.as_ref().expose_secret()
    });

    let response = app.post_verify_2fa(&two_fa_request).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Should be able to login with valid 2FA credentials"
    );

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_code_used_twice(app: &mut TestApp) {
    let email = get_random_email();
    let parsed_email = Email::parse(Secret::new(email.clone())).unwrap();
    let password = "password";

    assert_eq!(
        app.post_signup(&serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": true
        }))
        .await
        .status()
        .as_u16(),
        201
    );

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;
    assert_eq!(login_response.status().as_u16(), 206);

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&parsed_email)
        .await
        .unwrap();

    let two_fa_request = serde_json::json!({
      "email": email,
      "loginAttemptId": login_attempt_id.as_ref().expose_secret(),
      "2FACode": two_fa_code.as_ref().expose_secret()
    });

    let response = app.post_verify_2fa(&two_fa_request).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Should be able to login with valid 2FA credentials"
    );

    let response = app.post_verify_2fa(&two_fa_request).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Code should not be able to be used twice"
    );
}
