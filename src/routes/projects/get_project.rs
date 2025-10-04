use axum::{extract::Query, extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::Deserialize;

use crate::{
    domain::{Project, ProjectAPIError, ProjectId},
    utils::auth::get_claims,
    AppState,
};

#[derive(Deserialize)]
pub struct GetProjectQueryParams {
    #[serde(rename = "projectId")]
    project_id: uuid::Uuid,
}

#[tracing::instrument(name = "Get project route handler", skip_all)]
pub async fn get_project(
    State(state): State<AppState>,
    jar: CookieJar,
    query_params: Query<GetProjectQueryParams>,
) -> Result<(StatusCode, CookieJar, Json<Project>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let project_id = ProjectId::new(query_params.project_id);

    let project = state
        .project_store
        .write()
        .await
        .get_project(&user_id, &project_id)
        .await
        .map_err(|e| ProjectAPIError::UnexpectedError(eyre!(e)))?;

    let response = Json(project);

    Ok((StatusCode::OK, jar, response))
}
