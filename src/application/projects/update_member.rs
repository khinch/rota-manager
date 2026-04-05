use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::{check_member_access, ApplicationError},
    domain::{Member, MemberId, MemberName, UserId},
};

#[tracing::instrument(name = "[Application] Update project member", skip_all)]
pub async fn update_member(
    project_store: &ProjectStoreType,
    user_id: UserId,
    member_id: MemberId,
    member_name: MemberName,
) -> Result<Member, ApplicationError> {
    let mut member =
        check_member_access(&project_store, &user_id, &member_id).await?;

    member.member_name = member_name;

    project_store
        .write()
        .await
        .update_member(&member)
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
        application::projects::{update_member, ApplicationError},
        data_stores::test_utils::*,
        domain::{MemberId, MemberName, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(test_init_hashmap_store()));
        store
    }

    #[tokio::test]
    async fn update_member_success() {
        let store = init_store();

        let test_params = [
            (U1_USER_ID, U1_P1_M1_MEMBER_ID),
            (U2_USER_ID, U2_P1_M1_MEMBER_ID),
            (U2_USER_ID, U2_P2_M2_MEMBER_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let member_id = MemberId::parse(params.1).unwrap();
            let member_name = MemberName::parse("Updated Member")
                .expect("failed to parse member name");

            let member = update_member(
                &store,
                user_id,
                member_id.clone(),
                member_name.clone(),
            )
            .await
            .expect("failed to update member");

            assert_eq!(&member.member_id, &member_id);
            assert_eq!(&member.member_name, &member_name);

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
    async fn update_non_existent_member_returns_error() {
        let store = init_store();

        let user_id = UserId::default();
        let member_id = MemberId::default();
        let member_name = MemberName::parse("Updated member")
            .expect("failed to parse member name");

        assert_eq!(
            update_member(&store, user_id, member_id.clone(), member_name)
                .await,
            Err(ApplicationError::MemberIDNotFound(member_id)),
        );
    }

    #[tokio::test]
    async fn update_member_owned_by_different_user_should_return_not_found() {
        let store = init_store();

        let test_params = [
            (EMPTY_PROJECT_USER_ID, U1_P1_M1_MEMBER_ID),
            (U1_USER_ID, U2_P1_M1_MEMBER_ID),
            (U1_USER_ID, U2_P2_M2_MEMBER_ID),
            (U2_USER_ID, U1_P1_M1_MEMBER_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let member_id = MemberId::parse(params.1).unwrap();
            let member_name = MemberName::parse("Updated Member")
                .expect("failed to parse member name");

            assert_eq!(
                update_member(
                    &store,
                    user_id,
                    member_id.clone(),
                    member_name.clone(),
                )
                .await,
                Err(ApplicationError::MemberIDNotFound(member_id))
            );
        }
    }
}
