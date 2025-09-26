use color_eyre::eyre::eyre;

use crate::{
    app_state::ProjectStoreType,
    domain::{ProjectId, UserId},
    ProjectAPIError,
};

#[tracing::instrument(name = "Check user permissions for project", skip_all)]
pub async fn check_permissions_for_project(
    project_store: &mut ProjectStoreType,
    user_id: &UserId,
    project_id: &ProjectId,
) -> Result<(), ProjectAPIError> {
    let user_projects = project_store
        .write()
        .await
        .get_project_list(&user_id)
        .await
        .map_err(|e| ProjectAPIError::UnexpectedError(eyre!(e)))?;

    let (_project_id, _project_name) = user_projects
        .iter()
        .find(|(id, _)| id == project_id)
        .ok_or(ProjectAPIError::IDNotFoundError(*project_id.as_ref()))?;

    Ok(())
}
