use axum::{extract::Query, extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    application::projects,
    domain::{Project, ProjectId},
    routes::projects::ProjectAPIError,
    utils::auth::get_claims,
    AppState,
};

#[derive(Deserialize)]
pub struct GetProjectQueryParams {
    #[serde(rename = "projectId")]
    project_id: uuid::Uuid,
}

#[tracing::instrument(name = "[Route handler] Get project", skip_all)]
pub async fn get_project(
    State(state): State<AppState>,
    jar: CookieJar,
    query_params: Query<GetProjectQueryParams>,
) -> Result<(StatusCode, CookieJar, Json<Project>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let project_id = ProjectId::new(query_params.project_id);

    let project =
        projects::get_project(&state.project_store, user_id, project_id)
            .await?;

    let response = Json(project);

    Ok((StatusCode::OK, jar, response))
}
