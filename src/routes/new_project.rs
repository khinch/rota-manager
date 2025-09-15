use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{ProjectAPIError, ProjectId, ProjectName},
    utils::auth::get_claims,
    AppState,
};

#[tracing::instrument(name = "Create new project route handler", skip_all)]
pub async fn new_project(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<NewProjectRequest>,
) -> (
    CookieJar,
    Result<(StatusCode, Json<NewProjectResponse>), ProjectAPIError>,
) {
    let user_id = match get_claims(&jar, &state.banned_token_store).await {
        Ok(claims) => claims.id,
        Err(e) => return (jar, Err(ProjectAPIError::UnexpectedError(e))), // TODO error handling needs overhaul
    };

    let project_id = ProjectId::default();
    let mut project_store = state.project_store.write().await;

    let project_name = match ProjectName::parse(&request.name) {
        Ok(name) => name,
        Err(e) => return (jar, Err(ProjectAPIError::UnexpectedError(e))),
    };

    match project_store
        .add_project(&user_id, &project_id, &project_name)
        .await
    {
        Ok(()) => (),
        Err(e) => return (jar, Err(ProjectAPIError::UnexpectedError(e))),
    }

    let response = Json(NewProjectResponse {
        id: project_id.as_ref().to_string(),
        name: project_name.as_ref().to_string(),
    });

    (jar, Ok((StatusCode::CREATED, response)))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NewProjectResponse {
    pub name: String,
    pub id: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct NewProjectRequest {
    pub name: String,
}
