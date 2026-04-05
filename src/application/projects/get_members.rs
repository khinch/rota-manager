use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::{check_project_access, ApplicationError},
    domain::{Member, ProjectId, ProjectStoreError, UserId},
};

#[tracing::instrument(name = "[Application] Get project member list", skip_all)]
pub async fn get_project_member_list(
    project_store: &ProjectStoreType,
    user_id: UserId,
    project_id: ProjectId,
) -> Result<Vec<Member>, ApplicationError> {
    check_project_access(&project_store, &user_id, &project_id).await?;

    let member_list = project_store
        .write()
        .await
        .get_members(&project_id)
        .await
        .map_err(|e| match e {
            ProjectStoreError::ProjectIDNotFound => {
                ApplicationError::ProjectIDNotFound(project_id)
            }

            e => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    Ok(member_list)
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::ProjectStoreType,
        application::projects::{get_project_member_list, ApplicationError},
        data_stores::test_utils::*,
        domain::{ProjectId, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(test_init_hashmap_store()));
        store
    }

    #[tokio::test]
    async fn get_member_list_success() {
        let store = init_store();

        let test_params = [
            (U1_USER_ID, U1_P1_PROJECT_ID, 1),
            (U2_USER_ID, U2_P1_PROJECT_ID, 2),
            (U2_USER_ID, U2_P2_PROJECT_ID, 2),
            (EMPTY_PROJECT_USER_ID, EMPTY_PROJECT_PROJECT_ID, 0),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let project_id = ProjectId::parse(params.1).unwrap();

            let member_list =
                get_project_member_list(&store, user_id, project_id.clone())
                    .await
                    .expect("failed to retrieve project members");

            assert_eq!(member_list.len(), params.2);
        }
    }

    #[tokio::test]
    async fn non_existent_project_ids_should_return_not_found() {
        let store = init_store();

        let user_id = UserId::parse(U1_USER_ID).unwrap();
        let project_id = ProjectId::default();

        assert_eq!(
            get_project_member_list(&store, user_id, project_id.clone()).await,
            Err(ApplicationError::ProjectIDNotFound(project_id))
        );
    }

    #[tokio::test]
    async fn project_ids_belonging_to_another_user_should_return_not_found() {
        let store = init_store();

        let test_params = [
            (U1_USER_ID, U2_P1_PROJECT_ID),
            (U1_USER_ID, U2_P2_PROJECT_ID),
            (U2_USER_ID, U1_P1_PROJECT_ID),
            (EMPTY_PROJECT_USER_ID, U1_P1_PROJECT_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let project_id = ProjectId::parse(params.1).unwrap();

            assert_eq!(
                get_project_member_list(&store, user_id, project_id.clone())
                    .await,
                Err(ApplicationError::ProjectIDNotFound(project_id))
            );
        }
    }
}
