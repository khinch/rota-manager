use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    application::projects::get_project_list as get_project_list_app,
    domain::{ProjectId, ProjectName},
    routes::projects::ProjectAPIError,
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

    let project_list =
        get_project_list_app(&state.project_store, user_id).await?;

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
