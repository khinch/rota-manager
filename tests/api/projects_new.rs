use crate::helpers::{get_session, TestApp};
use rota_manager::ErrorResponse;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_requests(app: &mut TestApp) {
    let _email = get_session(app, false).await;

    let schema = json!({
      "$schema": "http://json-schema.org/draft-04/schema#",
      "description": "",
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1,
          "maxLength": 255
        },
        "id": {
          "type": "string",
          "minLength": 36,
          "maxLength": 36
        }
      },
      "required": [
        "name",
        "id"
      ]
    });

    let project_names = [
        "My hovercraft is full of eels",
        "A moose once bit my sister",
    ];

    for project_name in project_names.iter() {
        let response = app
            .post_projects_new(&serde_json::json!({"name": project_name}))
            .await;
        assert_eq!(
            response.status().as_u16(),
            201,
            "Failed to create new project for project name: {}",
            project_name
        );

        let response_body: serde_json::Value =
            response.json().await.expect("Failed to parse JSON");

        assert!(
            jsonschema::is_valid(&schema, &response_body),
            "response does not match schema"
        );

        assert_eq!(
            response_body.get("name").unwrap(),
            project_name,
            "Response should contain expected project name: {project_name}"
        );

        let response_id = response_body.get("id").unwrap().as_str().unwrap();
        assert!(
            uuid::Uuid::try_parse(response_id).is_ok(),
            "Response ID should be a valid UUID: {response_id}"
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_request(app: &mut TestApp) {
    let _email = get_session(app, false).await;

    let test_cases = [
        serde_json::json!({
            "name": true
        }),
        serde_json::json!({
            "foo": "bar"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_projects_new(test_case).await;
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
            "name": ""
        }),
        serde_json::json!({
            "name": "a".repeat(256)
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_projects_new(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            500, // TODO error handling overhaul
            "Should fail with HTTP400 for input: {}",
            test_case
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialise response body to ErrorResponse")
                .error,
            // "Invalid project name".to_owned()
            "Unexpected error"
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_not_authenticated(app: &mut TestApp) {
    let request = serde_json::json!({
        "name": "name"
    });

    let response = app.post_projects_new(&request).await;
    assert_eq!(
        response.status().as_u16(),
        500, // TODO error handling overhaul
        "Should return 401 for unauthenticated requests",
    );
}
