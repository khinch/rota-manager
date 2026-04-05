use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::{check_member_access, ApplicationError},
    domain::{Day, MemberId, Minute, Shift, UserId},
};

#[tracing::instrument(name = "[Application] Add shift to project", skip_all)]
pub async fn add_shift(
    project_store: &ProjectStoreType,
    user_id: UserId,
    member_id: MemberId,
    day: Day,
    start_time: Minute,
    end_time: Minute,
) -> Result<Shift, ApplicationError> {
    check_member_access(&project_store, &user_id, &member_id).await?;

    let shift = Shift::new(member_id, day, start_time, end_time)?;

    project_store
        .write()
        .await
        .add_shift(&shift)
        .await
        .map_err(|e| ApplicationError::UnexpectedError(eyre!(e)))?;
    Ok(shift)
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::ProjectStoreType,
        application::projects::{add_shift, ApplicationError},
        data_stores::test_utils::*,
        domain::{Day, MemberId, Minute, UserId},
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn init_store() -> ProjectStoreType {
        let store: ProjectStoreType =
            Arc::new(RwLock::new(test_init_hashmap_store()));
        store
    }

    #[tokio::test]
    async fn add_shift_success() {
        let store = init_store();

        let test_params = [
            (U1_USER_ID, U1_P1_M1_MEMBER_ID),
            (U2_USER_ID, U2_P2_M2_MEMBER_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let member_id = MemberId::parse(params.1).unwrap();
            let day = Day::Sunday;
            let start_time = Minute::parse(0).unwrap();
            let end_time = Minute::parse(1).unwrap();

            let shift = add_shift(
                &store,
                user_id,
                member_id.clone(),
                day,
                start_time.clone(),
                end_time.clone(),
            )
            .await
            .expect("failed to add shift to project");

            assert!(
                uuid::Uuid::parse_str(&shift.id.as_ref().to_string()).is_ok()
            );
            assert_eq!(&shift.member_id, &member_id);
            assert_eq!(&shift.day, &day);
            assert_eq!(&shift.start_time, &start_time);
            assert_eq!(&shift.end_time, &end_time);
        }
    }

    #[tokio::test]
    async fn adding_shift_to_non_existent_member_returns_not_found() {
        let store = init_store();

        let user_id = UserId::parse(U1_USER_ID).unwrap();
        let member_id = MemberId::default();
        let day = Day::Sunday;
        let start_time = Minute::parse(0).unwrap();
        let end_time = Minute::parse(1).unwrap();

        assert_eq!(
            add_shift(
                &store,
                user_id,
                member_id.clone(),
                day,
                start_time,
                end_time,
            )
            .await,
            Err(ApplicationError::MemberIDNotFound(member_id))
        );
    }

    #[tokio::test]
    async fn adding_shift_to_member_owned_by_another_user_returns_not_found() {
        let store = init_store();

        let test_params = [
            (U1_USER_ID, U2_P1_M1_MEMBER_ID),
            (U2_USER_ID, U1_P1_M1_MEMBER_ID),
            (EMPTY_PROJECT_USER_ID, U1_P1_M1_MEMBER_ID),
        ];

        for params in test_params {
            let user_id = UserId::parse(params.0).unwrap();
            let member_id = MemberId::parse(params.1).unwrap();
            let day = Day::Sunday;
            let start_time = Minute::parse(0).unwrap();
            let end_time = Minute::parse(1).unwrap();

            assert_eq!(
                add_shift(
                    &store,
                    user_id,
                    member_id.clone(),
                    day,
                    start_time,
                    end_time,
                )
                .await,
                Err(ApplicationError::MemberIDNotFound(member_id))
            );
        }
    }
}
