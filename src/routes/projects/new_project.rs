use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
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
) -> Result<(StatusCode, CookieJar, Json<NewProjectResponse>), ProjectAPIError>
{
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let project_id = ProjectId::default();
    let project_name = ProjectName::parse(&request.name)?;

    state
        .project_store
        .write()
        .await
        .add_project(&user_id, &project_id, &project_name)
        .await
        .map_err(|e| ProjectAPIError::UnexpectedError(eyre!(e)))?;

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
