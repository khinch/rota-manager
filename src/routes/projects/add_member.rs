use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::application::projects::{self};
use crate::routes::projects::ProjectAPIError;
use crate::{
    domain::{MemberName, ProjectId},
    utils::auth::get_claims,
    AppState,
};

#[derive(Debug, PartialEq, Serialize)]
pub struct AddMemberResponse {
    #[serde(rename = "projectId")]
    pub project_id: uuid::Uuid,
    #[serde(rename = "memberId")]
    pub member_id: uuid::Uuid,
    #[serde(rename = "memberName")]
    pub member_name: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct AddMemberRequest {
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "memberName")]
    pub member_name: String,
}

#[tracing::instrument(name = "[Route handler] Add member to project", skip_all)]
pub async fn add_member(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<AddMemberRequest>,
) -> Result<(StatusCode, CookieJar, Json<AddMemberResponse>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let project_id = ProjectId::parse(&request.project_id)?;
    let member_name = MemberName::parse(&request.member_name)?;

    let member = projects::add_member(
        &state.project_store,
        user_id,
        project_id.clone(),
        member_name,
    )
    .await?;

    let response = Json(AddMemberResponse {
        project_id: *member.project_id.as_ref(),
        member_id: *member.member_id.as_ref(),
        member_name: member.member_name.as_ref().to_owned(),
    });

    Ok((StatusCode::CREATED, jar, response))
}
