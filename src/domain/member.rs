use super::{MemberName, ProjectId};
use crate::id;

id!(MemberId);

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Member {
    pub project_id: ProjectId,
    pub member_id: MemberId,
    pub member_name: MemberName,
}

impl Member {
    pub fn new(project_id: ProjectId, member_name: MemberName) -> Self {
        Self {
            project_id,
            member_id: MemberId::default(),
            member_name,
        }
    }
}
