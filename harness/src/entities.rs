use sqlx::{Row, Sqlite, SqlitePool, Transaction, sqlite::SqliteRow};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

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
    /// References to other entities this entity was derived from or relates to.
    /// Loaded separately from the entity_references join table.
    pub references: Vec<Uuid>,
    pub created_at: String,
}

/// Row type for entities without references (loaded from a bare SELECT).
/// References are fetched separately and attached via `with_references`.
struct EntityRow {
    id: Uuid,
    project_id: Uuid,
    entity_type: EntityType,
    content: serde_json::Value,
    created_at: String,
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for EntityRow {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.try_get("id")?;
        let project_id: Uuid = row.try_get("project_id")?;
        let entity_type_str: String = row.try_get("entity_type")?;
        let content_json: String = row.try_get("content_json")?;
        let created_at: String = row.try_get("created_at")?;

        let entity_type = EntityType::from_db_str(&entity_type_str)
            .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::other(e.to_string()))))?;

        let content: serde_json::Value =
            serde_json::from_str(&content_json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(EntityRow {
            id,
            project_id,
            entity_type,
            content,
            created_at,
        })
    }
}

impl EntityRow {
    fn into_entity(self, references: Vec<Uuid>) -> Entity {
        Entity {
            id: self.id,
            project_id: self.project_id,
            entity_type: self.entity_type,
            content: self.content,
            references,
            created_at: self.created_at,
        }
    }
}

/// Load the reference UUIDs for a single entity.
async fn load_references(pool: &SqlitePool, entity_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(include_str!("entities/get_references.sql"))
        .bind(entity_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// Insert reference rows for an entity inside an existing transaction.
async fn insert_references(
    tx: &mut Transaction<'_, Sqlite>,
    source_id: Uuid,
    references: &[Uuid],
) -> Result<(), AppError> {
    for &target_id in references {
        sqlx::query(include_str!("entities/insert_references.sql"))
            .bind(source_id)
            .bind(target_id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

/// Create an entity and its references within an existing transaction.
/// Use this when the caller already holds a transaction (e.g. `projects::create`).
pub async fn create_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: Uuid,
    entity_type: EntityType,
    content: serde_json::Value,
    references: Vec<Uuid>,
) -> Result<Entity, AppError> {
    let id = Uuid::new_v4();
    let entity_type_str = entity_type.as_db_str();
    let content_json =
        serde_json::to_string(&content).map_err(|e| AppError::Internal(eyre::Report::from(e)))?;

    let row = sqlx::query_as::<_, EntityRow>(include_str!("entities/create.sql"))
        .bind(id)
        .bind(project_id)
        .bind(entity_type_str)
        .bind(&content_json)
        .fetch_one(&mut **tx)
        .await?;

    insert_references(tx, id, &references).await?;
    Ok(row.into_entity(references))
}

/// Create an entity and its references, managing the transaction internally.
pub async fn create(
    pool: &SqlitePool,
    project_id: Uuid,
    entity_type: EntityType,
    content: serde_json::Value,
    references: Vec<Uuid>,
) -> Result<Entity, AppError> {
    let mut tx = pool.begin().await?;
    let entity = create_in_tx(&mut tx, project_id, entity_type, content, references).await?;
    tx.commit().await?;
    Ok(entity)
}

pub async fn get(pool: &SqlitePool, id: Uuid) -> Result<Option<Entity>, AppError> {
    let row = sqlx::query_as::<_, EntityRow>(include_str!("entities/get.sql"))
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        None => Ok(None),
        Some(r) => {
            let refs = load_references(pool, r.id).await?;
            Ok(Some(r.into_entity(refs)))
        }
    }
}

pub async fn list(pool: &SqlitePool, limit: i64, offset: i64) -> Result<Page<Entity>, AppError> {
    let total: i64 = sqlx::query_scalar(include_str!("entities/count.sql"))
        .fetch_one(pool)
        .await?;

    let rows = sqlx::query_as::<_, EntityRow>(include_str!("entities/list.sql"))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let refs = load_references(pool, row.id).await?;
        items.push(row.into_entity(refs));
    }

    Ok(Page {
        items,
        total,
        limit,
        offset,
    })
}

pub async fn list_by_project(
    pool: &SqlitePool,
    project_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Page<Entity>, AppError> {
    let total: i64 = sqlx::query_scalar(include_str!("entities/count_by_project.sql"))
        .bind(project_id)
        .fetch_one(pool)
        .await?;

    let rows = sqlx::query_as::<_, EntityRow>(include_str!("entities/list_by_project.sql"))
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let refs = load_references(pool, row.id).await?;
        items.push(row.into_entity(refs));
    }

    Ok(Page {
        items,
        total,
        limit,
        offset,
    })
}
