use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::entities::{self, EntityType};
use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description_entity_id: Uuid,
    pub created_at: String,
}

/// Create a project with a description stored as a Raw entity.
pub async fn create(pool: &SqlitePool, name: &str, description: &str) -> Result<Project, AppError> {
    let mut tx = pool.begin().await?;
    let project = create_in_tx(&mut tx, name, description).await?;
    tx.commit().await?;
    Ok(project)
}

/// Used by callers that manage their own transaction.
pub async fn create_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    name: &str,
    description: &str,
) -> Result<Project, AppError> {
    let project_id = Uuid::new_v4();

    // Step 1: insert stub — description_entity_id nullable until step 3
    sqlx::query(include_str!("projects/insert_stub.sql"))
        .bind(project_id)
        .bind(name)
        .execute(&mut **tx)
        .await?;

    // Step 2: create description entity (project row now exists, FK satisfied)
    let entity = entities::create(
        &mut **tx,
        project_id,
        EntityType::Raw,
        serde_json::json!({ "text": description }),
        vec![],
    )
    .await?;

    // Step 3: set description_entity_id, return final row
    let project = sqlx::query_as::<_, Project>(include_str!("projects/create.sql"))
        .bind(entity.id)
        .bind(project_id)
        .fetch_one(&mut **tx)
        .await?;

    Ok(project)
}

pub async fn get(pool: &SqlitePool, id: Uuid) -> Result<Option<Project>, AppError> {
    let project = sqlx::query_as::<_, Project>(include_str!("projects/get.sql"))
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(project)
}

pub async fn exists(pool: &SqlitePool, id: Uuid) -> Result<bool, AppError> {
    let row: Option<(i64,)> = sqlx::query_as(include_str!("projects/exists.sql"))
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Project>, AppError> {
    let projects = sqlx::query_as::<_, Project>(include_str!("projects/list.sql"))
        .fetch_all(pool)
        .await?;
    Ok(projects)
}
