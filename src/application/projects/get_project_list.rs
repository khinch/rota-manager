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
        application::projects::new_project,
        data_stores::HashMapProjectStore,
        domain::{ProjectId, ProjectName, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(HashMapProjectStore::default()));
        store
    }

    #[tokio::test]
    async fn test_empty_list() {
        let store = init_store();

        let user_id = UserId::default();
        let project_list = store
            .write()
            .await
            .get_project_list(&user_id)
            .await
            .unwrap();
        assert!(project_list.is_empty());
    }

    #[tokio::test]
    async fn test_only_owned_projects_are_returned() {
        let store = init_store();

        let user_id_one = UserId::default();
        let project_name_one = ProjectName::parse("My Project").unwrap();
        let user_id_two = UserId::default();

        let _project = store
            .write()
            .await
            .add_project(&user_id_one, &ProjectId::default(), &project_name_one)
            .await
            .unwrap();

        let project_list_one = store
            .write()
            .await
            .get_project_list(&user_id_one)
            .await
            .unwrap();
        assert_eq!(project_list_one.len(), 1);

        let project_list_two = store
            .write()
            .await
            .get_project_list(&user_id_two)
            .await
            .unwrap();
        assert!(project_list_two.is_empty());
    }

    #[tokio::test]
    async fn test_projects_are_faithfully_returned() {
        let store = init_store();

        let user_id = UserId::default();
        let project_name_one = ProjectName::parse("My Project One").unwrap();
        let project_name_two = ProjectName::parse("My Project Two").unwrap();
        let project_id_one = ProjectId::default();
        let project_id_two = ProjectId::default();

        let _project = store
            .write()
            .await
            .add_project(&user_id, &project_id_one, &project_name_one)
            .await
            .unwrap();

        let _project = store
            .write()
            .await
            .add_project(&user_id, &project_id_two, &project_name_two)
            .await
            .unwrap();

        let project_list = store
            .write()
            .await
            .get_project_list(&user_id)
            .await
            .unwrap();

        assert_eq!(project_list.len(), 2);
        assert_eq!(project_list[0].0, project_id_one);
        assert_eq!(project_list[0].1, project_name_one);
        assert_eq!(project_list[1].0, project_id_two);
        assert_eq!(project_list[1].1, project_name_two);
    }
}
