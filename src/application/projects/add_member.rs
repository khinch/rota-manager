use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::ApplicationError,
    domain::{Member, MemberName, ProjectId, UserId},
};

#[tracing::instrument(name = "Add member to project application ", skip_all)]
pub async fn add_member(
    project_store: &ProjectStoreType,
    user_id: UserId,
    project_id: ProjectId,
    member_name: MemberName,
) -> Result<Member, ApplicationError> {
    project_store
        .write()
        .await
        .get_project_list(&user_id)
        .await
        .map_err(|e| ApplicationError::UnexpectedError(eyre!(e)))?
        .iter()
        .find(|(id, _)| id == &project_id)
        .ok_or(ApplicationError::ProjectIDNotFound(project_id.clone()))?;

    let member = Member::new(project_id, member_name);

    project_store
        .write()
        .await
        .add_member(&member)
        .await
        .map_err(|e| match e {
            e => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    Ok(member)
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::ProjectStoreType,
        application::projects::{add_member, ApplicationError},
        data_stores::HashMapProjectStore,
        domain::{MemberName, ProjectId, ProjectName, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(HashMapProjectStore::default()));
        store
    }

    #[tokio::test]
    async fn test_add_member_success() {
        let store = init_store();

        let user_id = UserId::default();
        let project_id = ProjectId::default();
        let project_name = ProjectName::parse("test_project")
            .expect("failed to parse project name");
        let member_name = MemberName::parse("test_member")
            .expect("failed to parse member name");

        store
            .write()
            .await
            .add_project(&user_id, &project_id, &project_name)
            .await
            .expect("failed to add project");

        add_member(&store, user_id, project_id, member_name)
            .await
            .expect("failed to add valid member to project");
    }

    #[tokio::test]
    async fn test_add_member_with_same_name_success() {
        let store = init_store();

        let user_id = UserId::default();
        let project_id = ProjectId::default();
        let project_name = ProjectName::parse("test_project")
            .expect("failed to parse project name");
        let member_name = MemberName::parse("test_member")
            .expect("failed to parse member name");

        store
            .write()
            .await
            .add_project(&user_id, &project_id, &project_name)
            .await
            .expect("failed to add project");

        add_member(
            &store,
            user_id.clone(),
            project_id.clone(),
            member_name.clone(),
        )
        .await
        .expect("failed to add member to project");

        add_member(&store, user_id, project_id, member_name)
            .await
            .expect("Failed adding member with same name");
    }

    #[tokio::test]
    async fn test_add_member_to_non_existent_project_should_error() {
        let store = init_store();

        let user_id = UserId::default();
        let project_id = ProjectId::default();
        let member_name = MemberName::parse("test_member")
            .expect("failed to parse member name");

        assert_eq!(
            add_member(&store, user_id, project_id.clone(), member_name).await,
            Err(ApplicationError::ProjectIDNotFound(project_id)),
        );
    }

    #[tokio::test]
    async fn test_add_member_to_project_owned_by_different_user() {
        let store = init_store();

        let user_id1 = UserId::default();
        let user_id2 = UserId::default();
        let project_id = ProjectId::default();
        let project_name = ProjectName::parse("test_project")
            .expect("failed to parse project name");
        let member_name = MemberName::parse("test_member")
            .expect("failed to parse member name");

        store
            .write()
            .await
            .add_project(&user_id1, &project_id, &project_name)
            .await
            .expect("failed to add project");

        assert_eq!(
            add_member(&store, user_id2, project_id.clone(), member_name).await,
            Err(ApplicationError::ProjectIDNotFound(project_id))
        );
    }
}
