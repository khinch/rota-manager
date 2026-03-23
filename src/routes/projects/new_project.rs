use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    application::projects::new_project as new_project_app,
    domain::{ProjectId, ProjectName},
    routes::projects::ProjectAPIError,
    utils::auth::get_claims,
    AppState,
};

#[tracing::instrument(name = "[Route handler] Create new project", skip_all)]
pub async fn new_project(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<NewProjectRequest>,
) -> Result<(StatusCode, CookieJar, Json<NewProjectResponse>), ProjectAPIError>
{
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let project_name = ProjectName::parse(&request.name)?;

    let project_id: ProjectId =
        new_project_app(&state.project_store, user_id, &project_name).await?;

    let response = Json(NewProjectResponse {
        id: project_id.as_ref().to_string(),
        name: project_name.as_ref().to_string(),
    });

    Ok((StatusCode::CREATED, jar, response))
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
