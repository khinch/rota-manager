use color_eyre::eyre::{eyre, Result};
use sqlx::PgPool;

use crate::domain::{
    ProjectId, ProjectName, ProjectStore, ProjectStoreError, UserId,
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
    ) -> Result<Vec<(ProjectId, ProjectName)>> {
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
    ) -> Result<()> {
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
}
