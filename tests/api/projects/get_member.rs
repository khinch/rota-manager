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
    let project_id = add_new_project(app, "Craggy Island").await;

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
        "id",
      ]
    });

    let members = ["Ted", "Dougal"];

    for member in members.iter() {
        let member_id = add_member(app, member, &project_id).await;
        let response = app.get_member(&member_id).await;

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

        assert_eq!(
            response_body.get("name").unwrap(),
            member,
            "Response should contain expected member name. Member: {}. Response: {}",
            member,
            response_body
        );

        assert_eq!(
            response_body.get("id").unwrap(),
            &member_id,
            "Response should contain expected memberId. MemberId: {}. Response: {}",
            &member_id,
            response_body
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_no_query_param(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;
    let _member_id = add_member(app, "Bar", &project_id).await;

    let response = app
        .http_client
        .get(format!("{}/projects/get-member", &app.address))
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
        "Failed to deserialize query string: missing field `memberId`"
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_400_if_invalid_query_param(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;
    let _member_id = add_member(app, "Bar", &project_id).await;

    let response = app.get_member("baz").await;

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
    let response = app.get_member("be9915f0-a4c2-48fb-977b-9f4f959c5729").await;
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

    let response = app.get_member(&member_id).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Failed to get member: {:?}",
        response
    );

    logout(app).await;
    let response = app.get_member(&member_id).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed to get member: {:?}",
        response
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_if_member_id_owned_by_someone_else(
    app: &mut TestApp,
) {
    let _session_one_email = get_session(app, false).await;
    let project_id_one = add_new_project(app, "Foo").await;
    let member_id_one = add_member(app, "Bar", &project_id_one).await;

    let response = app.get_member(&member_id_one).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "Should be able to get the member that was just added",
    );

    let _session_two_email = get_session(app, false).await;

    let response = app.get_member(&member_id_one).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for member IDs owned by someone else",
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_404_for_non_existent_member_id(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let _project_id = add_new_project(app, "Foo").await;

    let response = app.get_member("d8bb2e67-9979-4534-a13f-198664290c44").await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for non-existent member IDs",
    );
}
