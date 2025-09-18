use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{ProjectAPIError, ProjectId, ProjectName},
    utils::auth::get_claims,
    AppState,
};

#[tracing::instrument(name = "Get project list route handler", skip_all)]
pub async fn get_project_list(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<ProjectListResponse>), ProjectAPIError>
{
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;

    let project_list = state
        .project_store
        .write()
        .await
        .get_project_list(&user_id)
        .await
        .map_err(|e| ProjectAPIError::UnexpectedError(eyre!(e)))?;

    let response = Json(ProjectListResponse {
        projects: project_list
            .into_iter()
            .map(|(id, name)| Project { id, name })
            .collect(),
    });

    Ok((StatusCode::OK, jar, response))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: ProjectName,
}
