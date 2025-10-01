use std::str::FromStr;

use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        Day, MemberId, Minute, ProjectAPIError, ProjectStoreError, Shift,
    },
    utils::auth::get_claims,
    AppState,
};

#[tracing::instrument(name = "Add shift to project route handler", skip_all)]
pub async fn add_shift(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<AddShiftRequest>,
) -> Result<(StatusCode, CookieJar, Json<AddShiftResponse>), ProjectAPIError> {
    let user_id = get_claims(&jar, &state.banned_token_store).await?.id;

    let member_id = MemberId::new(request.member_id);
    let day = Day::from_str(&request.day)?;
    let start_time = Minute::parse(request.start_time)?;
    let end_time = Minute::parse(request.end_time)?;
    let shift = Shift::new(member_id, day, start_time, end_time)?;

    state
        .project_store
        .write()
        .await
        .add_shift(&user_id, &shift)
        .await
        .map_err(|e| match e {
            ProjectStoreError::MemberIDNotFound => {
                ProjectAPIError::IDNotFoundError(*shift.member_id.as_ref())
            }
            e => ProjectAPIError::UnexpectedError(eyre!(e)),
        })?;

    let response = Json(AddShiftResponse {
        id: *shift.id.as_ref(),
        member_id: *shift.member_id.as_ref(),
        day: shift.day.to_string(),
        start_time: shift.start_time.value_of(),
        end_time: shift.end_time.value_of(),
    });

    Ok((StatusCode::CREATED, jar, response))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AddShiftResponse {
    #[serde(rename = "id")]
    pub id: uuid::Uuid,
    #[serde(rename = "memberId")]
    pub member_id: uuid::Uuid,
    pub day: String,
    #[serde(rename = "startTime")]
    pub start_time: i16,
    #[serde(rename = "endTime")]
    pub end_time: i16,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct AddShiftRequest {
    #[serde(rename = "memberId")]
    pub member_id: uuid::Uuid,
    pub day: String,
    #[serde(rename = "startTime")]
    pub start_time: i16,
    #[serde(rename = "endTime")]
    pub end_time: i16,
}
