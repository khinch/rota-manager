use crate::helpers::{
    add_new_project, get_json_response_body, get_session, logout, TestApp,
};
use rota_manager::ErrorResponse;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_requests(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Craggy Island").await;

    let schema = json!({
      "$schema": "http://json-schema.org/draft-04/schema#",
      "description": "",
      "type": "object",
      "properties": {
        "memberName": {
          "type": "string",
          "minLength": 1,
          "maxLength": 255
        },
        "memberId": {
          "type": "string",
          "minLength": 36,
          "maxLength": 36
        },
        "projectId": {
          "type": "string",
          "minLength": 36,
          "maxLength": 36
        }
      },
      "required": [
        "memberName",
        "memberId",
        "projectId"
      ]
    });

    let members = ["Ted", "Dougal"];

    for member in members.iter() {
        let response = app
            .post_add_member(&serde_json::json!(
            {
                "memberName": member,
                "projectId": project_id
            }))
            .await;

        assert_eq!(
            response.status().as_u16(),
            201,
            "Failed to create new member: {}",
            member
        );

        let response_body = get_json_response_body(response).await;

        assert!(
            jsonschema::is_valid(&schema, &response_body),
            "response does not match schema"
        );

        assert_eq!(
            response_body.get("memberName").unwrap(),
            member,
            "Response should contain expected member name. Member: {}. Response: {}",
            member,
            response_body
        );

        assert_eq!(
            response_body.get("projectId").unwrap(),
            &project_id,
            "Response should contain expected projectId. ProjectId: {}. Response: {}",
            &project_id,
            response_body
        );

        let member_id =
            response_body.get("memberId").unwrap().as_str().unwrap();
        assert!(
            uuid::Uuid::try_parse(member_id).is_ok(),
            "Member ID should be a valid UUID: {member_id}"
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_request(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;

    let test_cases = [
        serde_json::json!({
            "projectId": project_id
        }),
        serde_json::json!({
            "memberName": "bar"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_add_member(test_case).await;
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
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;

    let test_cases = [
        (
            serde_json::json!({
                "memberName": "",
                "projectId": project_id
            }),
            "Validation error: Member name cannot be empty",
        ),
        (
            serde_json::json!({
                "memberName": "a".repeat(256),
                "projectId": project_id
            }),
            "Validation error: Max name length is 255 characters",
        ),
        (
            serde_json::json!({
                "memberName": "foo",
                "projectId": "ge9915f0-a4c2-48fb-977b-9f4f959c5729"
            }),
            "Validation error: Invalid project ID: failed to parse a UUID",
        ),
    ];

    for (body, expected_error) in test_cases.iter() {
        let response = app.post_add_member(body).await;
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
            expected_error.to_string()
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_not_authenticated(app: &mut TestApp) {
    let request = serde_json::json!(
        {
            "memberName": "foo",
            "projectId": "be9915f0-a4c2-48fb-977b-9f4f959c5729"
        }
    );

    let response = app.post_add_member(&request).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Should return 401 for unauthenticated requests",
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_logged_out(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;

    let request = serde_json::json!(
        {
            "memberName": "foo",
            "projectId": project_id
        }
    );

    let response = app.post_add_member(&request).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Should return 201 for valid requests",
    );

    logout(app).await;

    let response = app.post_add_member(&request).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Should return 401 for unauthenticated requests",
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_if_project_id_owned_by_someone_else(
    app: &mut TestApp,
) {
    let _session_one_email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;

    let request = serde_json::json!(
        {
            "memberName": "foo",
            "projectId": project_id
        }
    );

    let response = app.post_add_member(&request).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Should return 201 for valid requests",
    );

    let _session_two_email = get_session(app, false).await;

    let response = app.post_add_member(&request).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for project IDs owned by someone else",
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_for_non_existent_project_id(app: &mut TestApp) {
    let _session_one_email = get_session(app, false).await;
    let _project_id = add_new_project(app, "Foo").await;

    let request = serde_json::json!(
        {
            "memberName": "foo",
            "projectId": "be9915f0-a4c2-48fb-977b-9f4f959c5729"
        }
    );

    let response = app.post_add_member(&request).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for non-existent project IDs",
    );
}
