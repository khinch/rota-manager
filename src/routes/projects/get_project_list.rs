use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
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
) -> (
    CookieJar,
    Result<(StatusCode, Json<ProjectListResponse>), ProjectAPIError>,
) {
    let user_id = match get_claims(&jar, &state.banned_token_store).await {
        Ok(claims) => claims.id,
        Err(e) => return (jar, Err(ProjectAPIError::UnexpectedError(e))), // TODO error handling needs overhaul
    };

    let mut project_store = state.project_store.write().await;

    let project_list = match project_store.get_project_list(&user_id).await {
        Ok(list) => list,
        Err(e) => return (jar, Err(ProjectAPIError::UnexpectedError(e))),
    };

    let response = Json(ProjectListResponse {
        projects: project_list
            .into_iter()
            .map(|(id, name)| Project { id, name })
            .collect(),
    });

    (jar, Ok((StatusCode::OK, response)))
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
