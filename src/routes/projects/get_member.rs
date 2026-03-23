use axum::{extract::Query, extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    application::projects::get_member as get_member_app, domain::MemberId,
    routes::projects::ProjectAPIError, utils::auth::get_claims, AppState,
};

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(rename = "memberId")]
    member_id: uuid::Uuid,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct MemberResponse {
    pub id: String,
    pub name: String,
}

#[tracing::instrument(name = "[Route handler] Get member", skip_all)]
pub async fn get_member(
    State(state): State<AppState>,
    jar: CookieJar,
    query_params: Query<QueryParams>,
) -> Result<(StatusCode, CookieJar, Json<MemberResponse>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    let member_id = MemberId::new(query_params.member_id);

    let member =
        get_member_app(&state.project_store, user_id, member_id).await?;

    let response = Json(MemberResponse {
        id: member.member_id.as_ref().to_string(),
        name: member.member_name.as_ref().to_owned(),
    });

    Ok((StatusCode::OK, jar, response))
}
