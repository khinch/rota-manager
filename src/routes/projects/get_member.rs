use axum::{extract::Query, extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{MemberId, ProjectAPIError, ProjectStoreError},
    utils::auth::get_claims,
    AppState,
};

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(rename = "memberId")]
    member_id: uuid::Uuid,
}

#[tracing::instrument(name = "Get member route handler", skip_all)]
pub async fn get_member(
    State(state): State<AppState>,
    jar: CookieJar,
    query_params: Query<QueryParams>,
) -> Result<(StatusCode, CookieJar, Json<MemberResponse>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;
    tracing::debug!("user_id: {}", user_id.as_ref().to_string(),);

    let member_id = MemberId::new(query_params.member_id);
    tracing::debug!("member_id: {}", member_id.as_ref().to_string());

    let member = state
        .project_store
        .write()
        .await
        .get_member(&user_id, &member_id)
        .await
        .map_err(|e| match e {
            ProjectStoreError::MemberIDNotFound => {
                ProjectAPIError::IDNotFoundError(*member_id.as_ref())
            }
            e => ProjectAPIError::UnexpectedError(eyre!(e)),
        })?;

    let response = Json(MemberResponse {
        id: member.member_id.as_ref().to_string(),
        name: member.member_name.as_ref().to_owned(),
    });

    Ok((StatusCode::OK, jar, response))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: String,
    pub name: String,
}
