use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::ApplicationError,
    domain::{ProjectId, ProjectName, UserId},
};

#[tracing::instrument(name = "Create new project application layer", skip_all)]
pub async fn new_project(
    project_store: &ProjectStoreType,
    user_id: UserId,
    project_name: &ProjectName,
) -> Result<ProjectId, ApplicationError> {
    let project_id = ProjectId::default();

    project_store
        .write()
        .await
        .add_project(&user_id, &project_id, project_name)
        .await
        .map_err(|e| match e {
            e => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    Ok(project_id)
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::ProjectStoreType,
        application::projects::new_project,
        data_stores::HashMapProjectStore,
        domain::{ProjectName, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(HashMapProjectStore::default()));
        store
    }

    #[tokio::test]
    async fn test_add_project_success() {
        let store = init_store();

        let user_id = UserId::default();
        let project_name = ProjectName::parse("test_project")
            .expect("failed to parse project name");

        let _project_id = new_project(&store, user_id, &project_name)
            .await
            .expect("failed to add new project");
    }
}
