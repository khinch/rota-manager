use crate::helpers::{
    add_new_project, get_json_response_body, get_session, TestApp,
};
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_empty_collection_if_no_projects(app: &mut TestApp) {
    let _email = get_session(app, false).await;

    let response = app.get_projects_list().await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get projects list"
    );

    let expected_body = json!({"projects": []});
    let actual_body = get_json_response_body(response).await;
    assert_eq!(actual_body, expected_body);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_valid_list_for_valid_requests(app: &mut TestApp) {
    let _email = get_session(app, true).await;

    let schema = json!({
      "$schema": "http://json-schema.org/draft-04/schema#",
      "description": "",
      "type": "object",
      "properties": {
        "projects": {
          "type": "array",
          "uniqueItems": true,
          "minItems": 1,
          "items": {
            "required": [
              "id",
              "name"
            ],
            "properties": {
              "id": {
                "type": "string",
                "minLength": 36,
                "maxLength": 36
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 255
              }
            }
          }
        }
      },
      "required": [
        "projects"
      ]
    });

    // Project 1
    let first_project_name = "My awesome first project";
    let first_project_id = add_new_project(app, first_project_name).await;

    let response = app.get_projects_list().await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get projects list: {:?}",
        response
    );

    let response_body = get_json_response_body(response).await;
    assert!(jsonschema::is_valid(&schema, &response_body));

    let expected_body = json!({
        "projects": [
            {
                "id": first_project_id,
                "name": first_project_name
            }
        ]
    });
    assert_eq!(response_body, expected_body);

    // Project 2
    let second_project_name = "My awesome second project";
    let second_project_id = add_new_project(app, second_project_name).await;

    let response = app.get_projects_list().await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get projects list: {:?}",
        response
    );

    let response_body = get_json_response_body(response).await;
    assert!(jsonschema::is_valid(&schema, &response_body));

    let expected_body = json!({
        "projects": [
            {
                "id": first_project_id,
                "name": first_project_name
            },
            {
                "id": second_project_id,
                "name": second_project_name
            }
        ]
    });
    assert_eq!(response_body, expected_body);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_not_authenticated(app: &mut TestApp) {
    let response = app.get_projects_list().await;
    assert_eq!(
        response.status().as_u16(),
        500, // TODO error handling overhaul
        "Should return 401 (actually 500 for now) for unauthenticated requests",
    );
}
