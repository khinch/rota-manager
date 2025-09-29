use crate::helpers::{
    add_member, add_new_project, get_json_response_body, get_session, logout,
    TestApp,
};
use rota_manager::ErrorResponse;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_requests(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Craggy Island").await;
    let member_id = add_member(app, "Ted", &project_id).await;

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

    let response = app
        .put_member(
            &member_id,
            &serde_json::json!(
            {
                "memberName": "Dougal",
            }),
        )
        .await;

    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to update member: {}",
        member_id
    );

    let response_body = get_json_response_body(response).await;

    assert!(
        jsonschema::is_valid(&schema, &response_body),
        "response does not match schema"
    );

    let expected_response = serde_json::json!({
        "memberId": &member_id,
        "projectId": &project_id,
        "memberName": "Dougal"
    });

    assert_eq!(response_body, expected_response);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_request(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;
    let member_id = add_member(app, "Bar", &project_id).await;

    let test_cases = [
        serde_json::json!({
            "memberNam": "Baz"
        }),
        serde_json::json!({
            "memberName": true
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.put_member(&member_id, &test_case).await;
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
async fn should_return_400_if_invalid_request_body(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;
    let member_id = add_member(app, "Bar", &project_id).await;

    let test_cases = [
        (
            serde_json::json!({
                "memberName": "",
            }),
            "Validation error: Member name cannot be empty",
        ),
        (
            serde_json::json!({
                "memberName": "a".repeat(256),
            }),
            "Validation error: Max name length is 255 characters",
        ),
    ];

    for (body, expected_error) in test_cases.iter() {
        let response = app.put_member(&member_id, &body).await;
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
async fn should_return_400_if_invalid_query_param(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;
    let _member_id = add_member(app, "Bar", &project_id).await;

    let response = app
        .put_member(
            "foo",
            &serde_json::json!(
            {
                "memberName": "Dougal",
            }),
        )
        .await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "Should be bad request for invalid query param: {:?}",
        response
    );

    let response_body = response.text().await.unwrap();
    assert_eq!(
        response_body,
        "Failed to deserialize query string: UUID parsing failed: invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `o` at 2"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_not_authenticated(app: &mut TestApp) {
    let id = "be9915f0-a4c2-48fb-977b-9f4f959c5729";

    let request = serde_json::json!(
        {
            "memberName": "foo",
            "projectId": id,
        }
    );

    let response = app.put_member(id, &request).await;
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
    let member_id = add_member(app, "Bar", &project_id).await;

    let request = serde_json::json!(
        {
            "memberName": "Baz",
        }
    );

    let response = app.put_member(&member_id, &request).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Should return 200 for valid requests",
    );

    logout(app).await;

    let response = app.put_member(&member_id, &request).await;
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
    let member_id = add_member(app, "Bar", &project_id).await;

    let request = serde_json::json!(
        {
            "memberName": "Baz",
        }
    );

    let response = app.put_member(&member_id, &request).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Should return 200 for valid requests",
    );

    let _session_two_email = get_session(app, false).await;

    let response = app.put_member(&member_id, &request).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for member IDs owned by someone else",
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_for_non_existent_project_id(app: &mut TestApp) {
    let _session_one_email = get_session(app, false).await;
    let _project_id = add_new_project(app, "Foo").await;
    let non_existent_member_id = "be9915f0-a4c2-48fb-977b-9f4f959c5729";

    let request = serde_json::json!(
        {
            "memberName": "Bar",

        }
    );

    let response = app.put_member(&non_existent_member_id, &request).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for non-existent member IDs",
    );
}
