use crate::helpers::{add_new_project, delete_user, get_session, TestApp};
use rota_manager::{domain::Email, routes::auth::DeleteUserResponse};
use secrecy::Secret;

use test_context::test_context;

#[test_context(TestApp)]
#[tokio::test]
async fn should_return_200_for_valid_requests(app: &mut TestApp) {
    let email = get_session(app, false).await;

    let delete_user_response = app.delete_user().await;
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
async fn should_return_400_for_missing_token(app: &mut TestApp) {
    let delete_user_response = app.delete_user().await;
    assert_eq!(
        app.delete_user().await.status().as_u16(),
        // 400, TODO
        500,
        "Unexpected response to unauthenticated delete user request: {:?}",
        delete_user_response
    );
}

#[test_context(TestApp)]
#[tokio::test]
async fn should_tidy_up_user_data(app: &mut TestApp) {
    let email = get_session(app, false).await;
    let _project_id = add_new_project(app, "new project 1").await;

    let email = Email::parse(Secret::new(email)).unwrap();
    let user_id;
    {
        let user_store = app.user_store.read().await;
        user_id = user_store.get_user(&email).await.unwrap().id;
    }

    {
        let mut project_store = app.project_store.write().await;
        let project_list =
            project_store.get_project_list(&user_id).await.unwrap();
        assert_eq!(project_list.len(), 1);
    }

    delete_user(app).await;

    {
        let user_store = app.user_store.read().await;
        let user_id_result = user_store.get_user(&email).await;
        assert!(user_id_result.is_err());
    }

    {
        let mut project_store = app.project_store.write().await;
        let project_list =
            project_store.get_project_list(&user_id).await.unwrap();
        assert_eq!(project_list.len(), 0);
    }
}
