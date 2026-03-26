use std::collections::{hash_map::Entry, HashMap};

use crate::domain::{
    Member, MemberId, Project, ProjectId, ProjectName, ProjectStore,
    ProjectStoreError, Shift, ShiftId, UserId,
};
use color_eyre::eyre::Result;

#[derive(Default)]
pub struct HashMapProjectStore {
    members: HashMap<MemberId, Member>,
    projects: HashMap<ProjectId, (UserId, ProjectName)>,
    shifts: HashMap<ShiftId, Shift>,
}

#[async_trait::async_trait]
impl ProjectStore for HashMapProjectStore {
    async fn get_project_list(
        &mut self,
        user_id: &UserId,
    ) -> Result<Vec<(ProjectId, ProjectName)>, ProjectStoreError> {
        let project_list = self
            .projects
            .iter()
            .filter(|(_, (uid, _))| uid == user_id)
            .map(|(pid, (_, pname))| (pid.clone(), pname.clone()))
            .collect();
        Ok(project_list)
    }

    async fn add_project(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
        project_name: &ProjectName,
    ) -> Result<(), ProjectStoreError> {
        match self.projects.entry(project_id.clone()) {
            Entry::Vacant(e) => {
                e.insert((user_id.clone(), project_name.clone()));
                ()
            }
            Entry::Occupied(_) => {
                return Err(ProjectStoreError::ProjectIDExists)
            }
        }

        Ok(())
    }

    async fn delete_projects(
        &mut self,
        _user_id: &UserId,
    ) -> Result<(), ProjectStoreError> {
        todo!()
    }
    async fn add_member(
        &mut self,
        member: &Member,
    ) -> Result<(), ProjectStoreError> {
        match self.members.entry(member.member_id.clone()) {
            Entry::Vacant(e) => {
                e.insert(member.clone());
                ()
            }
            Entry::Occupied(_) => {
                return Err(ProjectStoreError::MemberIDExists)
            }
        }

        Ok(())
    }
    async fn get_member(
        &mut self,
        member_id: &MemberId,
    ) -> Result<Member, ProjectStoreError> {
        match self.members.get(member_id) {
            Some(member) => Ok(member.clone()),
            None => Err(ProjectStoreError::MemberIDNotFound),
        }
    }
    async fn update_member(
        &mut self,
        _user_id: &UserId,
        _member: &Member,
    ) -> Result<(), ProjectStoreError> {
        todo!()
    }
    async fn get_members(
        &mut self,
        _user_id: &UserId,
        _project_id: &ProjectId,
    ) -> Result<Vec<Member>, ProjectStoreError> {
        todo!()
    }
    async fn delete_members(
        &mut self,
        _user_id: &UserId,
        _project_id: &ProjectId,
    ) -> Result<(), ProjectStoreError> {
        todo!()
    }
    async fn add_shift(
        &mut self,
        shift: &Shift,
    ) -> Result<(), ProjectStoreError> {
        match self.shifts.entry(shift.id.clone()) {
            Entry::Vacant(e) => {
                e.insert(shift.clone());
                ()
            }
            Entry::Occupied(_) => return Err(ProjectStoreError::ShiftIdExists),
        }

        Ok(())
    }
    async fn get_project(
        &mut self,
        _user_id: &UserId,
        _project_id: &ProjectId,
    ) -> Result<Project, ProjectStoreError> {
        todo!()
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::domain::{
        Member, MemberId, MemberName, ProjectId, ProjectName, UserId,
    };

    use super::HashMapProjectStore;

    pub const EMPTY_PROJECT_USER_ID: &str =
        "00000000-0000-0000-0000-000000000000";
    pub const EMPTY_PROJECT_PROJECT_ID: &str =
        "11111111-0000-0000-0000-000000000000";
    pub const EMPTY_PROJECT_PROJECT_NAME: &str = "Empty Project";

    pub const U1_USER_ID: &str = "00000000-1111-1111-1111-111111111111";
    pub const U1_P1_PROJECT_ID: &str = "11111111-1111-1111-1111-111111111111";
    pub const U1_P1_PROJECT_NAME: &str = "Test Project";
    pub const U1_P1_M1_MEMBER_ID: &str = "22222222-1111-1111-1111-111111111111";
    pub const U1_P1_M1_MEMBER_NAME: &str = "Test Member";

    pub const U2_USER_ID: &str = "00000000-2222-2222-2222-222222222222";
    pub const U2_P1_PROJECT_ID: &str = "11111111-2222-1111-1111-111111111111";
    pub const U2_P1_PROJECT_NAME: &str = "Test Project One";
    pub const U2_P1_M1_MEMBER_ID: &str = "22222222-2222-1111-1111-111111111111";
    pub const U2_P1_M1_MEMBER_NAME: &str = "Test Project One - Member One";
    pub const U2_P1_M2_MEMBER_ID: &str = "22222222-2222-1111-1111-222222222222";
    pub const U2_P1_M2_MEMBER_NAME: &str = "Test Project One - Member Two";
    pub const U2_P2_PROJECT_ID: &str = "11111111-2222-2222-2222-222222222222";
    pub const U2_P2_PROJECT_NAME: &str = "Test Project Two";
    pub const U2_P2_M1_MEMBER_ID: &str = "22222222-2222-2222-2222-111111111111";
    pub const U2_P2_M1_MEMBER_NAME: &str = "Test Project Two - Member One";
    pub const U2_P2_M2_MEMBER_ID: &str = "22222222-2222-2222-2222-222222222222";
    pub const U2_P2_M2_MEMBER_NAME: &str = "Test Project Two - Member Two";

    /// Initiates a store pre-populated with data for testing
    ///
    /// *************
    /// Empty Project
    /// *************
    /// user_id    00000000-0000-0000-0000-000000000000
    /// project_id 11111111-0000-0000-0000-000000000000 Empty Project
    /// No members
    ///
    /// ******
    /// User 1
    /// ******
    /// user_id    00000000-1111-1111-1111-111111111111
    /// project_id 11111111-1111-1111-1111-111111111111 Test Project
    /// member_id  22222222-1111-1111-1111-111111111111 Test Member
    ///
    /// ******
    /// User 2
    /// ******
    /// user_id    00000000-2222-2222-2222-222222222222
    /// project_id 11111111-2222-1111-1111-111111111111 Test Project One
    /// member_id  22222222-2222-1111-1111-111111111111 Test Project One - Member One
    /// member_id  22222222-2222-1111-1111-222222222222 Test Project One - Member Two
    /// project_id 11111111-2222-2222-2222-222222222222 Test Project Two
    /// member_id  22222222-2222-2222-2222-111111111111 Test Project Two - Member One
    /// member_id  22222222-2222-2222-2222-222222222222 Test Project Two - Member Two
    pub fn test_init_hashmap_store() -> HashMapProjectStore {
        let mut store = HashMapProjectStore::default();

        // Empty project
        store.projects.insert(
            ProjectId::parse(EMPTY_PROJECT_PROJECT_ID)
                .expect("failed to parse UUID"),
            (
                UserId::parse(EMPTY_PROJECT_USER_ID)
                    .expect("failed to parse UUID"),
                ProjectName::parse(EMPTY_PROJECT_PROJECT_NAME)
                    .expect("failed to parse project name"),
            ),
        );

        // User one with one project containing a single member
        store.projects.insert(
            ProjectId::parse(U1_P1_PROJECT_ID).expect("failed to parse UUID"),
            (
                UserId::parse(U1_USER_ID).expect("failed to parse UUID"),
                ProjectName::parse(U1_P1_PROJECT_NAME)
                    .expect("failed to parse project name"),
            ),
        );

        store.members.insert(
            MemberId::parse(U1_P1_M1_MEMBER_ID).unwrap(),
            Member {
                project_id: ProjectId::parse(U1_P1_PROJECT_ID).unwrap(),
                member_id: MemberId::parse(U1_P1_M1_MEMBER_ID).unwrap(),
                member_name: MemberName::parse(U1_P1_M1_MEMBER_NAME).unwrap(),
            },
        );

        // User two, with two projects, two members in each
        store.projects.insert(
            ProjectId::parse(U2_P1_PROJECT_ID).expect("failed to parse UUID"),
            (
                UserId::parse(U2_USER_ID).expect("failed to parse UUID"),
                ProjectName::parse(U2_P1_PROJECT_NAME)
                    .expect("failed to parse project name"),
            ),
        );

        store.members.insert(
            MemberId::parse(U2_P1_M1_MEMBER_ID).unwrap(),
            Member {
                project_id: ProjectId::parse(U2_P1_PROJECT_ID).unwrap(),
                member_id: MemberId::parse(U2_P1_M1_MEMBER_ID).unwrap(),
                member_name: MemberName::parse(U2_P1_M1_MEMBER_NAME).unwrap(),
            },
        );

        store.members.insert(
            MemberId::parse(U2_P1_M2_MEMBER_ID).unwrap(),
            Member {
                project_id: ProjectId::parse(U2_P1_PROJECT_ID).unwrap(),
                member_id: MemberId::parse(U2_P1_M2_MEMBER_ID).unwrap(),
                member_name: MemberName::parse(U2_P1_M2_MEMBER_NAME).unwrap(),
            },
        );

        store.projects.insert(
            ProjectId::parse(U2_P2_PROJECT_ID).expect("failed to parse UUID"),
            (
                UserId::parse(U2_USER_ID).expect("failed to parse UUID"),
                ProjectName::parse(U2_P2_PROJECT_NAME)
                    .expect("failed to parse project name"),
            ),
        );

        store.members.insert(
            MemberId::parse(U2_P2_M1_MEMBER_ID).unwrap(),
            Member {
                project_id: ProjectId::parse(U2_P2_PROJECT_ID).unwrap(),
                member_id: MemberId::parse(U2_P2_M1_MEMBER_ID).unwrap(),
                member_name: MemberName::parse(U2_P2_M1_MEMBER_NAME).unwrap(),
            },
        );

        store.members.insert(
            MemberId::parse(U2_P2_M2_MEMBER_ID).unwrap(),
            Member {
                project_id: ProjectId::parse(U2_P2_PROJECT_ID).unwrap(),
                member_id: MemberId::parse(U2_P2_M2_MEMBER_ID).unwrap(),
                member_name: MemberName::parse(U2_P2_M2_MEMBER_NAME).unwrap(),
            },
        );

        return store;
    }
}
