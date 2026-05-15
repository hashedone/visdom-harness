use sqlx::{Row, Sqlite, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Raw,
    Knowledge,
    Summary,
}

impl EntityType {
    fn as_db_str(&self) -> &'static str {
        match self {
            EntityType::Raw => "raw",
            EntityType::Knowledge => "knowledge",
            EntityType::Summary => "summary",
        }
    }

    fn from_db_str(s: &str) -> Result<Self, AppError> {
        match s {
            "raw" => Ok(EntityType::Raw),
            "knowledge" => Ok(EntityType::Knowledge),
            "summary" => Ok(EntityType::Summary),
            other => Err(AppError::Internal(eyre::eyre!(
                "unknown entity_type: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub project_id: Uuid,
    pub entity_type: EntityType,
    pub content: serde_json::Value,
    pub contributing_entity_ids: Vec<Uuid>,
    pub created_at: String,
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for Entity {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.try_get("id")?;
        let project_id: Uuid = row.try_get("project_id")?;
        let entity_type_str: String = row.try_get("entity_type")?;
        let content_json: String = row.try_get("content_json")?;
        let contributing_json: String = row.try_get("contributing_entity_ids_json")?;
        let created_at: String = row.try_get("created_at")?;

        let entity_type = EntityType::from_db_str(&entity_type_str)
            .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::other(e.to_string()))))?;

        let content: serde_json::Value =
            serde_json::from_str(&content_json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let contributing_entity_ids: Vec<Uuid> = serde_json::from_str(&contributing_json)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Entity {
            id,
            project_id,
            entity_type,
            content,
            contributing_entity_ids,
            created_at,
        })
    }
}

pub async fn create<'e, E>(
    executor: E,
    project_id: Uuid,
    entity_type: EntityType,
    content: serde_json::Value,
    contributing_entity_ids: Vec<Uuid>,
) -> Result<Entity, AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let id = Uuid::new_v4();
    let entity_type_str = entity_type.as_db_str();
    let content_json =
        serde_json::to_string(&content).map_err(|e| AppError::Internal(eyre::Report::from(e)))?;
    let contributing_json = serde_json::to_string(&contributing_entity_ids)
        .map_err(|e| AppError::Internal(eyre::Report::from(e)))?;

    let entity = sqlx::query_as::<_, Entity>(include_str!("entities/create.sql"))
        .bind(id)
        .bind(project_id)
        .bind(entity_type_str)
        .bind(&content_json)
        .bind(&contributing_json)
        .fetch_one(executor)
        .await?;

    Ok(entity)
}

pub async fn get(pool: &SqlitePool, id: Uuid) -> Result<Option<Entity>, AppError> {
    let entity = sqlx::query_as::<_, Entity>(include_str!("entities/get.sql"))
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(entity)
}

pub async fn list(pool: &SqlitePool, limit: i64) -> Result<Vec<Entity>, AppError> {
    let entities = sqlx::query_as::<_, Entity>(include_str!("entities/list.sql"))
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(entities)
}

pub async fn list_by_project(
    pool: &SqlitePool,
    project_id: Uuid,
    limit: i64,
) -> Result<Vec<Entity>, AppError> {
    let entities = sqlx::query_as::<_, Entity>(include_str!("entities/list_by_project.sql"))
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(entities)
}
