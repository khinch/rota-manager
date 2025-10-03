use serde::{Deserialize, Serialize};

use crate::domain::{ProjectName, Shift};

use super::{MemberId, MemberName, ProjectId};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "projectId")]
    pub project_id: ProjectId,
    #[serde(rename = "projectName")]
    pub project_name: ProjectName,
    pub members: Vec<ProjectMember>,
}

impl Project {
    pub fn new(
        project_id: ProjectId,
        project_name: ProjectName,
        members: Vec<ProjectMember>,
    ) -> Self {
        Self {
            project_id,
            project_name,
            members,
        }
    }
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub struct ProjectMember {
    #[serde(rename = "memberId")]
    pub member_id: MemberId,
    #[serde(rename = "memberName")]
    pub member_name: MemberName,
    pub shifts: Vec<Shift>,
}

impl ProjectMember {
    pub fn new(
        member_id: MemberId,
        member_name: MemberName,
        shifts: Vec<Shift>,
    ) -> Self {
        Self {
            member_id,
            member_name,
            shifts,
        }
    }
}
