use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    application::projects,
    domain::{MemberId, MemberName},
    routes::projects::ProjectAPIError,
    utils::auth::get_claims,
    AppState,
};

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(rename = "memberId")]
    member_id: uuid::Uuid,
}

#[tracing::instrument(name = "[Route handler] Update member", skip_all)]
pub async fn update_member(
    State(state): State<AppState>,
    jar: CookieJar,
    query_params: Query<QueryParams>,
    Json(request): Json<UpdateMemberRequest>,
) -> Result<(StatusCode, CookieJar, Json<UpdateMemberResponse>), ProjectAPIError>
{
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let member_id = MemberId::new(query_params.member_id);
    let member_name = MemberName::parse(&request.member_name)?;

    let member = projects::update_member(
        &state.project_store,
        user_id,
        member_id,
        member_name,
    )
    .await?;

    let response = Json(UpdateMemberResponse {
        project_id: *member.project_id.as_ref(),
        member_id: *member.member_id.as_ref(),
        member_name: member.member_name.as_ref().to_owned(),
    });

    Ok((StatusCode::OK, jar, response))
}

#[derive(Debug, PartialEq, Serialize)]
pub struct UpdateMemberResponse {
    #[serde(rename = "projectId")]
    pub project_id: uuid::Uuid,
    #[serde(rename = "memberId")]
    pub member_id: uuid::Uuid,
    #[serde(rename = "memberName")]
    pub member_name: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct UpdateMemberRequest {
    #[serde(rename = "memberName")]
    pub member_name: String,
}
