use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        Member, MemberName, ProjectAPIError, ProjectId, ProjectStoreError,
    },
    utils::auth::get_claims,
    AppState,
};

#[tracing::instrument(name = "Add member to project route handler", skip_all)]
pub async fn add_member(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<AddMemberRequest>,
) -> Result<(StatusCode, CookieJar, Json<AddMemberResponse>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;

    let project_id = ProjectId::parse(&request.project_id)?;

    let member_name = MemberName::parse(request.member_name)?;
    let member = Member::new(project_id, member_name);

    state
        .project_store
        .write()
        .await
        .add_member(&user_id, &member)
        .await
        .map_err(|e| match e {
            ProjectStoreError::ProjectIDNotFound => {
                ProjectAPIError::IDNotFoundError(*member.project_id.as_ref())
            }
            e => ProjectAPIError::UnexpectedError(eyre!(e)),
        })?;

    let response = Json(AddMemberResponse {
        project_id: *member.project_id.as_ref(),
        member_id: *member.member_id.as_ref(),
        member_name: member.member_name.as_ref().to_owned(),
    });

    Ok((StatusCode::CREATED, jar, response))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
