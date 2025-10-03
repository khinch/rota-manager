use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    Day, Member, MemberId, MemberName, Minute, Project, ProjectId,
    ProjectMember, ProjectName, ProjectStore, ProjectStoreError, Shift,
    ShiftId, UserId,
};

pub struct PostgresProjectStore {
    pool: PgPool,
}

impl PostgresProjectStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ProjectStore for PostgresProjectStore {
    #[tracing::instrument(
        name = "Getting project list from PostgreSQL",
        skip_all
    )]
    async fn get_project_list(
        &mut self,
        user_id: &UserId,
    ) -> Result<Vec<(ProjectId, ProjectName)>, ProjectStoreError> {
        let rows = sqlx::query!(
            r#"
                    SELECT project_id, project_name
                    FROM projects_list
                    WHERE user_id = $1
                    "#,
            user_id.as_ref()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            err => ProjectStoreError::UnexpectedError(err.into()),
        })?;

        rows.into_iter()
            .map(|row| {
                let project_id = ProjectId::new(row.project_id);
                let project_name = ProjectName::parse(&row.project_name)
                    .map_err(|e| {
                        ProjectStoreError::UnexpectedError(eyre!(e))
                    })?;
                Ok((project_id, project_name))
            })
            .collect()
    }

    #[tracing::instrument(name = "Adding project to PostgreSQL", skip_all)]
    async fn add_project(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
        project_name: &ProjectName,
    ) -> Result<(), ProjectStoreError> {
        sqlx::query!(
            r#"
            INSERT INTO projects_list (user_id, project_id, project_name) VALUES ($1, $2, $3)
            "#,
            user_id.as_ref() as &uuid::Uuid,
            project_id.as_ref() as &uuid::Uuid,
            project_name.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ProjectStoreError::ProjectIDExists
            }
            err => ProjectStoreError::UnexpectedError(err.into()),
        })?;
        Ok(())
    }

    #[tracing::instrument(name = "Deleting all projects for user", skip_all)]
    async fn delete_projects(
        &mut self,
        user_id: &UserId,
    ) -> Result<(), ProjectStoreError> {
        sqlx::query!(
            r#"
                   DELETE FROM projects_list WHERE user_id = $1
                   "#,
            user_id.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?;

        Ok(())
    }

    #[tracing::instrument(name = "Adding member to PostgreSQL", skip_all)]
    async fn add_member(
        &mut self,
        user_id: &UserId,
        member: &Member,
    ) -> Result<(), ProjectStoreError> {
        self.get_project_list(&user_id)
            .await
            .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?
            .iter()
            .find(|(id, _)| id == &member.project_id)
            .ok_or(ProjectStoreError::ProjectIDNotFound)?;

        sqlx::query!(
            r#"
            INSERT INTO members (member_id, project_id, member_name) VALUES ($1, $2, $3)
            "#,
            member.member_id.as_ref() as &uuid::Uuid,
            member.project_id.as_ref() as &uuid::Uuid,
            member.member_name.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ProjectStoreError::MemberIDExists
            }
            e => ProjectStoreError::UnexpectedError(eyre!(e)),
        })?;
        Ok(())
    }

    #[tracing::instrument(name = "Getting member from PostgreSQL", skip_all)]
    async fn get_member(
        &mut self,
        user_id: &UserId,
        member_id: &MemberId,
    ) -> Result<Member, ProjectStoreError> {
        sqlx::query!(
            r#"
                SELECT members.project_id, members.member_id, members.member_name
                FROM members
                INNER JOIN projects_list ON members.project_id = projects_list.project_id
                WHERE members.member_id = $1 AND projects_list.user_id = $2
            "#,
            member_id.as_ref(),
            user_id.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ProjectStoreError::MemberIDNotFound,
            e => ProjectStoreError::UnexpectedError(eyre!(e)),
        })
        .map(|row| {
            Ok(Member {
                project_id: ProjectId::new(row.project_id),
                member_id: MemberId::new(row.member_id),
                member_name: MemberName::parse(row.member_name.to_owned())
                    .map_err(|e| {
                        ProjectStoreError::UnexpectedError(eyre!(e))
                    })?,
            })
        })?
    }

    #[tracing::instrument(name = "Updating member in PostgreSQL", skip_all)]
    async fn update_member(
        &mut self,
        user_id: &UserId,
        member: &Member,
    ) -> Result<(), ProjectStoreError> {
        self.get_project_list(&user_id)
            .await
            .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?
            .iter()
            .find(|(id, _)| id == &member.project_id)
            .ok_or(ProjectStoreError::ProjectIDNotFound)?;

        sqlx::query!(
            r#"
            UPDATE members SET member_name = $2
            WHERE member_id = $1
            "#,
            member.member_id.as_ref() as &uuid::Uuid,
            member.member_name.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ProjectStoreError::MemberIDNotFound,
            e => ProjectStoreError::UnexpectedError(eyre!(e)),
        })?;
        Ok(())
    }

    #[tracing::instrument(name = "Getting members from PostgreSQL", skip_all)]
    async fn get_members(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> Result<Vec<Member>, ProjectStoreError> {
        self.get_project_list(user_id)
            .await
            .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?
            .iter()
            .find(|(id, _)| id == project_id)
            .ok_or(ProjectStoreError::ProjectIDNotFound)?;

        let rows = sqlx::query!(
            r#"
                SELECT project_id, member_id, member_name
                FROM members
                WHERE project_id = $1
            "#,
            project_id.as_ref()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ProjectStoreError::MemberIDNotFound,
            e => ProjectStoreError::UnexpectedError(eyre!(e)),
        })?;

        rows.into_iter()
            .map(|row| {
                let member = Member {
                    project_id: ProjectId::new(row.project_id),
                    member_id: MemberId::new(row.member_id),
                    member_name: MemberName::parse(row.member_name.to_owned())
                        .map_err(|e| {
                            ProjectStoreError::UnexpectedError(eyre!(e))
                        })?,
                };
                Ok(member)
            })
            .collect()
    }

    #[tracing::instrument(name = "Deleting all members for project", skip_all)]
    async fn delete_members(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> Result<(), ProjectStoreError> {
        self.get_project_list(&user_id)
            .await
            .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?
            .iter()
            .find(|(id, _)| id == project_id)
            .ok_or(ProjectStoreError::ProjectIDNotFound)?;

        sqlx::query!(
            r#"
                DELETE FROM members WHERE project_id = $1
            "#,
            project_id.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?;

        Ok(())
    }

    #[tracing::instrument(name = "Adding shift to PostgreSQL", skip_all)]
    async fn add_shift(
        &mut self,
        user_id: &UserId,
        shift: &Shift,
    ) -> Result<(), ProjectStoreError> {
        let _member = self.get_member(&user_id, &shift.member_id).await?;

        sqlx::query!(
            r#"
            INSERT INTO shifts (id, member_id, day, in_time, out_time) VALUES ($1, $2, $3, $4, $5)
            "#,
            shift.id.as_ref() as &uuid::Uuid,
            shift.member_id.as_ref() as &uuid::Uuid,
            shift.day as i16,
            shift.start_time.value_of(),
            shift.end_time.value_of()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ProjectStoreError::ShiftIdExists
            }
            e => ProjectStoreError::UnexpectedError(eyre!(e)),
        })?;
        Ok(())
    }

    #[tracing::instrument(
        name = "Getting project details from PostreSQL",
        skip_all
    )]
    async fn get_project(
        &mut self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> Result<Project, ProjectStoreError> {
        let project_row = sqlx::query!(
            r#"
            SELECT project_id, project_name
            FROM projects_list
            WHERE project_id = $1
            AND user_id = $2
            "#,
            project_id.as_ref(),
            user_id.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ProjectStoreError::ProjectIDNotFound,
            err => ProjectStoreError::UnexpectedError(eyre!(err)),
        })?;

        let member_rows = sqlx::query!(
            r#"
                SELECT member_id, member_name
                FROM members
                WHERE project_id = $1
            "#,
            project_id.as_ref()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?;

        let mut member_map = HashMap::<uuid::Uuid, ProjectMember>::new();
        for row in member_rows {
            let member_id = MemberId::new(row.member_id);
            let member_name = MemberName::parse(row.member_name)
                .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?;
            member_map.insert(
                member_id.as_ref().to_owned(),
                ProjectMember {
                    member_id,
                    member_name,
                    shifts: Vec::new(),
                },
            );
        }

        let member_ids: Vec<Uuid> =
            member_map.keys().map(|id| *id.as_ref()).collect();
        if !member_ids.is_empty() {
            let shift_rows = sqlx::query!(
                r#"
                    SELECT id, member_id, day, in_time, out_time
                    FROM shifts
                    WHERE member_id = ANY($1)
               "#,
                &member_ids
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?;

            for row in shift_rows {
                let member_id = MemberId::new(row.member_id);
                if let Some(member) = member_map.get_mut(&member_id.as_ref()) {
                    let shift = Shift {
                        id: ShiftId::new(row.id),
                        member_id: member_id.clone(),
                        day: Day::try_from(row.day).map_err(|e| {
                            ProjectStoreError::UnexpectedError(eyre!(e))
                        })?,
                        start_time: Minute::parse(row.in_time).map_err(
                            |e| ProjectStoreError::UnexpectedError(eyre!(e)),
                        )?,
                        end_time: Minute::parse(row.out_time).map_err(|e| {
                            ProjectStoreError::UnexpectedError(eyre!(e))
                        })?,
                    };
                    member.shifts.push(shift);
                }
            }
        }

        let project = Project {
            project_id: ProjectId::new(project_row.project_id),
            project_name: ProjectName::parse(&project_row.project_name)
                .map_err(|e| ProjectStoreError::UnexpectedError(eyre!(e)))?,
            members: member_map.into_values().collect(),
        };

        Ok(project)
    }
}
