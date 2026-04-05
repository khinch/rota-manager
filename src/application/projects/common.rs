use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    application::projects::ApplicationError,
    domain::{Member, MemberId, ProjectId, ProjectStoreError, UserId},
};

#[tracing::instrument(
    name = "[Application] Check user access to member",
    skip_all
)]
pub async fn check_member_access(
    project_store: &ProjectStoreType,
    user_id: &UserId,
    member_id: &MemberId,
) -> Result<Member, ApplicationError> {
    let member = project_store
        .write()
        .await
        .get_member(member_id)
        .await
        .map_err(|e| match e {
            ProjectStoreError::MemberIDNotFound => {
                ApplicationError::MemberIDNotFound(member_id.to_owned())
            }
            _ => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    check_project_access(project_store, user_id, &member.project_id)
        .await
        .map_err(|e| match e {
            ApplicationError::ProjectIDNotFound(_) => {
                ApplicationError::MemberIDNotFound(member_id.to_owned())
            }
            _ => ApplicationError::UnexpectedError(eyre!(e)),
        })?;

    Ok(member)
}

#[tracing::instrument(
    name = "[Application] Check user access to project",
    skip_all
)]
pub async fn check_project_access(
    project_store: &ProjectStoreType,
    user_id: &UserId,
    project_id: &ProjectId,
) -> Result<(), ApplicationError> {
    project_store
        .write()
        .await
        .get_project_list(user_id)
        .await
        .map_err(|e| ApplicationError::UnexpectedError(eyre!(e)))?
        .iter()
        .find(|(id, _)| id == project_id)
        .ok_or(ApplicationError::ProjectIDNotFound(project_id.to_owned()))?;
    Ok(())
}
