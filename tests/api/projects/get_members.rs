use crate::helpers::{
    add_member, add_new_project, get_json_response_body, get_session, logout,
    TestApp,
};

use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_requests(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Barry Island").await;

    let schema = json!({
      "$schema": "http://json-schema.org/draft-04/schema#",
      "description": "",
      "type": "object",
      "properties": {
        "projectId": {
          "type": "string",
          "minLength": 36,
          "maxLength": 36
        },
        "members": {
          "type": "array",
          "items": {
            "type": "object",
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
            },
          }
        },
      },
      "required": [
        "projectId",
        "members",
      ]
    });

    let response = app.get_members(&project_id).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get member: {:?}",
        response
    );

    let response_body = get_json_response_body(response).await;

    assert!(
        jsonschema::is_valid(&schema, &response_body),
        "response does not match schema"
    );

    let expected_response_body = json!({
        "projectId": &project_id,
        "members": []
    });

    assert_eq!(response_body, expected_response_body);

    let members = ["Ted", "Dougal"];
    let mut member_ids: Vec<String> = vec![];

    for member in members.iter() {
        member_ids.push(add_member(app, member, &project_id).await);
    }

    let response = app.get_members(&project_id).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get member: {:?}",
        response
    );

    let response_body = get_json_response_body(response).await;

    assert!(
        jsonschema::is_valid(&schema, &response_body),
        "response does not match schema"
    );

    let expected_response_body = json!({
        "projectId": &project_id,
        "members": [
            {"id": member_ids[0], "name": members[0]},
            {"id": member_ids[1], "name": members[1]}
        ]
    });

    assert_eq!(response_body, expected_response_body);
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_no_query_param(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let _project_id = add_new_project(app, "Foo").await;

    let response = app
        .http_client
        .get(format!("{}/projects/get-members", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        400,
        "Failed to get member: {:?}",
        response
    );

    let response_body = response.text().await.unwrap();
    assert_eq!(
        response_body,
        "Failed to deserialize query string: missing field `projectId`"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_invalid_query_param(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let _project_id = add_new_project(app, "Foo").await;

    let response = app.get_members("baz").await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "Failed to get member: {:?}",
        response
    );

    let response_body = response.text().await.unwrap();
    assert_eq!(
        response_body,
        "Failed to deserialize query string: UUID parsing failed: invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `z` at 3"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_401_if_not_authenticated(app: &mut TestApp) {
    let response = app
        .get_members("be9915f0-a4c2-48fb-977b-9f4f959c5729")
        .await;
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
    let _member_id = add_member(app, "Bar", &project_id).await;

    let response = app.get_members(&project_id).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get member: {:?}",
        response
    );

    logout(app).await;
    let response = app.get_members(&project_id).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed to get member: {:?}",
        response
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_if_project_id_owned_by_someone_else(
    app: &mut TestApp,
) {
    let _session_one_email = get_session(app, false).await;
    let project_id_one = add_new_project(app, "Foo").await;

    let response = app.get_members(&project_id_one).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Should be able to get the current (empty) member list",
    );

    let _session_two_email = get_session(app, false).await;

    let response = app.get_members(&project_id_one).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for project IDs owned by someone else",
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_for_non_existent_member_id(app: &mut TestApp) {
    let _email = get_session(app, false).await;

    let response = app
        .get_members("60d8e457-8934-48ce-9d16-f503bd7ef085")
        .await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for non-existent project IDs",
    );
}
