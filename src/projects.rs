use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

pub async fn create(pool: &SqlitePool, name: &str, description: &str) -> Result<Project, AppError> {
    let id = Uuid::new_v4().to_string();
    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, name, description) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?;
    Ok(project)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Project>, AppError> {
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(project)
}

pub async fn exists(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM projects WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}
