use crate::helpers::{
    add_member, add_new_project, get_json_response_body, get_session, logout,
    TestApp,
};
use rota_manager::ErrorResponse;
use serde_json::json;
use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_201_for_valid_requests(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Craggy Island").await;
    let member_id = add_member(app, "Ted", &project_id).await;

    let schema = json!({
      "$schema": "http://json-schema.org/draft-04/schema#",
      "description": "",
      "type": "object",
      "properties": {
        "id": {
          "type": "string",
          "minLength": 36,
          "maxLength": 36
        },
        "memberId": {
          "type": "string",
          "minLength": 36,
          "maxLength": 36
        },
        "day": {
          "type": "string",
          "enum": ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
        },
        "startTime": {
          "type": "number",
          "min": 0,
          "max": 1440
        },
        "endTime": {
          "type": "number",
          "min": 0,
          "max": 1440
        }
      },
      "required": [
        "id",
        "memberId",
        "day",
        "startTime",
        "endTime"
      ]
    });

    let requests = [
        &json!(
        {
            "memberId": &member_id,
            "day": "Sunday",
            "startTime": 0,
            "endTime": 1440
        }),
        &json!(
        {
            "memberId": &member_id,
            "day": "Saturday",
            "startTime": 0,
            "endTime": 1
        }),
    ];

    for request in requests.iter() {
        let response = app.post_shift(&request).await;

        assert_eq!(
            response.status().as_u16(),
            201,
            "Failed to create new shift: {}",
            request
        );

        let response_body = get_json_response_body(response).await;

        assert!(
            jsonschema::is_valid(&schema, &response_body),
            "response does not match schema"
        );

        assert!(uuid::Uuid::try_parse(
            response_body.get("id").unwrap().as_str().unwrap()
        )
        .is_ok());

        assert_eq!(
            response_body.get("memberId").unwrap(),
            &member_id,
            "Response should contain expected memberId. MemberId: {}. Response: {}",
            &member_id,
            response_body
        );

        assert_eq!(
            response_body.get("startTime").unwrap(),
            request.get("startTime").unwrap(),
        );

        assert_eq!(
            response_body.get("endTime").unwrap(),
            request.get("endTime").unwrap(),
        );

        assert_eq!(
            response_body.get("day").unwrap(),
            request.get("day").unwrap(),
        );
    }
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_422_if_malformed_request(app: &mut TestApp) {
    let _email = get_session(app, false).await;
    let project_id = add_new_project(app, "Foo").await;
    let member_id = add_member(app, "Bar", &project_id).await;

    let test_cases = [
        &json!(
        {
            "memberId": &member_id,
            "day": "Sunday",
            "startTime": 0,
        }),
        &json!(
        {
            "memberId": &member_id,
            "day": "Saturday",
            "endTime": 1
        }),
        &json!(
        {
            "memberId": &member_id,
            "startTime": 0,
            "endTime": 1
        }),
        &json!(
        {
            "day": "Saturday",
            "startTime": 0,
            "endTime": 1
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_shift(test_case).await;
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
    let member_id = add_member(app, "Bar", &project_id).await;

    let test_cases = [
        (
            &json!({
                "memberId": &member_id,
                "day": "Sunday",
                "startTime": 0,
                "endTime": 1441
            }),
            "Validation error: Minute cannot be after midnight",
        ),
        (
            &json!({
                "memberId": &member_id,
                "day": "Sunday",
                "startTime": -1,
                "endTime": 1440
            }),
            "Validation error: Minute cannot be before midnight",
        ),
        (
            &json!({
                "memberId": &member_id,
                "day": "Sunday",
                "startTime": 1440,
                "endTime": 0
            }),
            "Validation error: Start time must be before end time",
        ),
        (
            &json!({
                "memberId": &member_id,
                "day": "Funday",
                "startTime": 1440,
                "endTime": 0
            }),
            "Validation error: Invalid day",
        ),
        (
            &json!({
                "memberId": &member_id,
                "day": "0",
                "startTime": 1440,
                "endTime": 0
            }),
            "Validation error: Invalid day",
        ),
    ];

    for (body, expected_error) in test_cases.iter() {
        let response = app.post_shift(body).await;
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
    let request = json!(
        {
            "memberId": "2a6af785-e170-4ab6-ac1f-691772640f31",
            "day": "Sunday",
            "startTime": 0,
            "endTime": 1440
        }
    );

    let response = app.post_shift(&request).await;
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

    let request = json!(
        {
            "memberId": &member_id,
            "day": "Sunday",
            "startTime": 0,
            "endTime": 1440
        }
    );

    let response = app.post_shift(&request).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Should return 201 for valid requests",
    );

    logout(app).await;

    let response = app.post_shift(&request).await;
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
            "memberId": &member_id,
            "day": "Sunday",
            "startTime": 0,
            "endTime": 1440
        }
    );

    let response = app.post_shift(&request).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Should return 201 for valid requests",
    );

    let _session_two_email = get_session(app, false).await;

    let response = app.post_shift(&request).await;
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
    let project_id = add_new_project(app, "Foo").await;
    let _member_id = add_member(app, "bar", &project_id).await;

    let request = serde_json::json!(
        {
            "memberId": "2a6af785-e170-4ab6-ac1f-691772640f31",
            "day": "Sunday",
            "startTime": 0,
            "endTime": 1440
        }
    );

    let response = app.post_shift(&request).await;
    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for non-existent project IDs",
    );
}
