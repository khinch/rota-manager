use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::{check_member_access, ApplicationError},
    domain::{Member, MemberId, ProjectStoreError, UserId},
};

#[tracing::instrument(name = "[Application] Get member", skip_all)]
pub async fn get_member(
    project_store: &ProjectStoreType,
    user_id: UserId,
    member_id: MemberId,
) -> Result<Member, ApplicationError> {
    check_member_access(project_store, &user_id, &member_id).await?;

    let member = project_store
        .write()
        .await
        .get_member(&member_id)
        .await
        .map_err(|e| match e {
            ProjectStoreError::MemberIDNotFound => {
                ApplicationError::MemberIDNotFound(member_id)
            }
            e => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    Ok(member)
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::ProjectStoreType,
        application::projects::{get_member, ApplicationError},
        data_stores::test_utils::*,
        domain::{MemberId, MemberName, ProjectId, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(test_init_hashmap_store()));
        store
    }

    #[tokio::test]
    async fn get_member_success() {
        let store = init_store();

        let test_params = [
            (
                U1_USER_ID,
                U1_P1_M1_MEMBER_ID,
                U1_P1_PROJECT_ID,
                U1_P1_M1_MEMBER_NAME,
            ),
            (
                U2_USER_ID,
                U2_P1_M2_MEMBER_ID,
                U2_P1_PROJECT_ID,
                U2_P1_M2_MEMBER_NAME,
            ),
            (
                U2_USER_ID,
                U2_P2_M1_MEMBER_ID,
                U2_P2_PROJECT_ID,
                U2_P2_M1_MEMBER_NAME,
            ),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let member_id = MemberId::parse(params.1).unwrap();

            let member = get_member(&store, user_id, member_id.clone())
                .await
                .unwrap();

            assert_eq!(member.project_id, ProjectId::parse(params.2).unwrap());
            assert_eq!(member.member_id, member_id);
            assert_eq!(
                member.member_name,
                MemberName::parse(params.3).unwrap()
            );
        }
    }

    #[tokio::test]
    async fn get_non_existent_member_returns_not_found_error() {
        let store = init_store();

        let user_ids = [EMPTY_PROJECT_USER_ID, U1_USER_ID, U2_USER_ID];

        for user_id in user_ids {
            let user_id = UserId::parse(user_id).unwrap();
            let member_id = MemberId::default();

            assert_eq!(
                get_member(&store, user_id, member_id.clone()).await,
                Err(ApplicationError::MemberIDNotFound(member_id))
            );
        }
    }

    #[tokio::test]
    async fn get_member_in_project_owned_by_different_user_returns_not_found_error(
    ) {
        let store = init_store();

        let test_params = [
            (EMPTY_PROJECT_USER_ID, U1_P1_M1_MEMBER_ID),
            (U1_USER_ID, U2_P2_M2_MEMBER_ID),
            (U2_USER_ID, U1_P1_M1_MEMBER_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let member_id = MemberId::parse(params.1).unwrap();

            assert_eq!(
                get_member(&store, user_id, member_id.clone()).await,
                Err(ApplicationError::MemberIDNotFound(member_id))
            );
        }
    }
}
