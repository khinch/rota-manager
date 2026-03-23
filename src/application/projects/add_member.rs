use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::{check_project_access, ApplicationError},
    domain::{Member, MemberName, ProjectId, UserId},
};

#[tracing::instrument(name = "[Application] Add member to project", skip_all)]
pub async fn add_member(
    project_store: &ProjectStoreType,
    user_id: UserId,
    project_id: ProjectId,
    member_name: MemberName,
) -> Result<Member, ApplicationError> {
    check_project_access(&project_store, &user_id, &project_id).await?;

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
        data_stores::test_utils::*,
        domain::{MemberName, ProjectId, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(test_init_hashmap_store()));
        store
    }

    #[tokio::test]
    async fn add_member_success() {
        let store = init_store();

        let test_params = [
            (EMPTY_PROJECT_USER_ID, EMPTY_PROJECT_PROJECT_ID),
            (U1_USER_ID, U1_P1_PROJECT_ID),
            (U2_USER_ID, U2_P1_PROJECT_ID),
            (U2_USER_ID, U2_P2_PROJECT_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let project_id = ProjectId::parse(params.1).unwrap();
            let member_name = MemberName::parse("New Member")
                .expect("failed to parse member name");

            let member = add_member(
                &store,
                user_id,
                project_id.clone(),
                member_name.clone(),
            )
            .await
            .expect("failed to add member to project");

            assert_eq!(&member.project_id, &project_id);
            assert_eq!(&member.member_name, &member_name);
            assert!(uuid::Uuid::parse_str(
                &member.member_id.as_ref().to_string()
            )
            .is_ok());

            assert_eq!(
                member,
                store
                    .write()
                    .await
                    .get_member(&member.member_id)
                    .await
                    .unwrap()
            );
        }
    }

    #[tokio::test]
    async fn add_member_to_non_existent_project_returns_error() {
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
    async fn add_member_to_project_owned_by_different_user() {
        let store = init_store();

        let user_id = UserId::parse(U1_USER_ID).unwrap();
        let project_id = ProjectId::parse(U2_P1_PROJECT_ID).unwrap();
        let member_name = MemberName::parse("test_member")
            .expect("failed to parse member name");

        assert_eq!(
            add_member(&store, user_id, project_id.clone(), member_name).await,
            Err(ApplicationError::ProjectIDNotFound(project_id))
        );
    }
}
