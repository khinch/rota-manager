use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::ApplicationError,
    domain::{ProjectId, ProjectName, UserId},
};

#[tracing::instrument(name = "Get project list application layer", skip_all)]
pub async fn get_project_list(
    project_store: &ProjectStoreType,
    user_id: UserId,
) -> Result<Vec<(ProjectId, ProjectName)>, ApplicationError> {
    let project_list = project_store
        .write()
        .await
        .get_project_list(&user_id)
        .await
        .map_err(|e| match e {
            e => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    Ok(project_list)
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::ProjectStoreType,
        application::projects::get_project_list,
        data_stores::test_utils::*,
        domain::{ProjectId, ProjectName, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(test_init_hashmap_store()));
        store
    }

    #[tokio::test]
    async fn test_empty_list() {
        let store = init_store();
        let user_id = UserId::default();
        let project_list = get_project_list(&store, user_id).await.unwrap();
        assert!(project_list.is_empty());
    }

    #[tokio::test]
    async fn test_only_owned_projects_are_returned() {
        let store = init_store();

        let project_list =
            get_project_list(&store, UserId::parse(U1_USER_ID).unwrap())
                .await
                .unwrap();

        assert_eq!(project_list.len(), 1);
        assert_eq!(
            project_list[0],
            (
                ProjectId::parse(U1_P1_PROJECT_ID).unwrap(),
                ProjectName::parse(U1_P1_PROJECT_NAME).unwrap()
            )
        );

        let project_list =
            get_project_list(&store, UserId::parse(U2_USER_ID).unwrap())
                .await
                .unwrap();

        assert_eq!(project_list.len(), 2);
        assert!(project_list.contains(&(
            ProjectId::parse(U2_P1_PROJECT_ID).unwrap(),
            ProjectName::parse(U2_P1_PROJECT_NAME).unwrap()
        )));
        assert!(project_list.contains(&(
            ProjectId::parse(U2_P2_PROJECT_ID).unwrap(),
            ProjectName::parse(U2_P2_PROJECT_NAME).unwrap()
        )));
    }

    #[tokio::test]
    async fn test_projects_are_accurately_returned() {
        let store = init_store();

        let project_list = store
            .write()
            .await
            .get_project_list(&UserId::parse(U2_USER_ID).unwrap())
            .await
            .unwrap();

        let project_one_id = ProjectId::parse(U2_P1_PROJECT_ID).unwrap();
        let project_two_id = ProjectId::parse(U2_P2_PROJECT_ID).unwrap();
        let project_one_name = ProjectName::parse(U2_P1_PROJECT_NAME).unwrap();
        let project_two_name = ProjectName::parse(U2_P2_PROJECT_NAME).unwrap();

        assert_eq!(project_list.len(), 2);
        assert_eq!(
            project_list.iter().find(|(id, _)| id == &project_one_id),
            Some(&(project_one_id, project_one_name))
        );
        assert_eq!(
            project_list.iter().find(|(id, _)| id == &project_two_id),
            Some(&(project_two_id, project_two_name))
        );
    }
}
