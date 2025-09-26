use axum::{extract::Query, extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{ProjectAPIError, ProjectId, ProjectStoreError},
    utils::auth::get_claims,
    AppState,
};

#[derive(Deserialize)]
pub struct GetMemberListQueryParams {
    #[serde(rename = "projectId")]
    project_id: uuid::Uuid,
}

#[tracing::instrument(name = "Get member list route handler", skip_all)]
pub async fn get_member_list_for_project(
    State(state): State<AppState>,
    jar: CookieJar,
    query_params: Query<GetMemberListQueryParams>,
) -> Result<(StatusCode, CookieJar, Json<MemberListResponse>), ProjectAPIError>
{
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    tracing::debug!("user_id: {}", user_id.as_ref().to_string(),);

    let project_id = ProjectId::new(query_params.project_id);
    tracing::debug!("project_id: {}", project_id.as_ref().to_string());

    let member_list = state
        .project_store
        .write()
        .await
        .get_members(&user_id, &project_id)
        .await
        .map_err(|e| match e {
            ProjectStoreError::ProjectIDNotFound => {
                ProjectAPIError::IDNotFoundError(*project_id.as_ref())
            }
            e => ProjectAPIError::UnexpectedError(eyre!(e)),
        })?;

    let response = Json(MemberListResponse {
        project_id,
        members: member_list
            .into_iter()
            .map(|member| Member {
                id: member.member_id.as_ref().to_string(),
                name: member.member_name.as_ref().to_owned(),
            })
            .collect(),
    });

    Ok((StatusCode::OK, jar, response))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberListResponse {
    #[serde(rename = "projectId")]
    pub project_id: ProjectId,
    pub members: Vec<Member>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub name: String,
}
