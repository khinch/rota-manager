use std::collections::{hash_map::Entry, HashMap};

use crate::domain::{
    Member, MemberId, Project, ProjectId, ProjectName, ProjectStore,
    ProjectStoreError, Shift, UserId,
};
use color_eyre::eyre::Result;

#[derive(Default)]
pub struct HashMapProjectStore {
    members: HashMap<MemberId, Member>,
    projects: HashMap<ProjectId, (UserId, ProjectName)>,
}

impl HashMapProjectStore {
    pub fn init() -> Self {
        let store = HashMapProjectStore::default();
        return store;
    }
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
        user_id: &UserId,
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
        user_id: &UserId,
        member_id: &MemberId,
    ) -> Result<Member, ProjectStoreError> {
        todo!()
    }
    async fn update_member(
        &mut self,
        user_id: &UserId,
        member: &Member,
    ) -> Result<(), ProjectStoreError> {
        todo!()
    }
    async fn get_members(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> Result<Vec<Member>, ProjectStoreError> {
        todo!()
    }
    async fn delete_members(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> Result<(), ProjectStoreError> {
        todo!()
    }
    async fn add_shift(
        &mut self,
        user_id: &UserId,
        shift: &Shift,
    ) -> Result<(), ProjectStoreError> {
        todo!()
    }
    async fn get_project(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> Result<Project, ProjectStoreError> {
        todo!()
    }
}
