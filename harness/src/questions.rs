use sqlx::SqlitePool;
use uuid::Uuid;

use crate::entities::{self, EntityType, Page};
use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Question {
    pub id: Uuid,
    pub project_id: Uuid,
    pub question: String,
    pub instructions: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Answer {
    pub id: Uuid,
    pub question_id: Uuid,
    pub entity_id: Uuid,
    pub received_at: String,
}

pub async fn create(
    pool: &SqlitePool,
    project_id: Uuid,
    question: &str,
    instructions: &str,
) -> Result<Question, AppError> {
    let id = Uuid::new_v4();
    let q = sqlx::query_as::<_, Question>(include_str!("questions/create.sql"))
        .bind(id)
        .bind(project_id)
        .bind(question)
        .bind(instructions)
        .fetch_one(pool)
        .await?;
    Ok(q)
}

pub async fn get(pool: &SqlitePool, id: Uuid) -> Result<Option<Question>, AppError> {
    let q = sqlx::query_as::<_, Question>(include_str!("questions/get.sql"))
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(q)
}

pub async fn list(
    pool: &SqlitePool,
    project_id: Uuid,
    status_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Page<Question>, AppError> {
    let (total, items) = match status_filter {
        Some(status) => {
            let total: i64 = sqlx::query_scalar(include_str!("questions/count_by_status.sql"))
                .bind(project_id)
                .bind(status)
                .fetch_one(pool)
                .await?;
            let items = sqlx::query_as::<_, Question>(include_str!("questions/list_by_status.sql"))
                .bind(project_id)
                .bind(status)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;
            (total, items)
        }
        None => {
            let total: i64 = sqlx::query_scalar(include_str!("questions/count.sql"))
                .bind(project_id)
                .fetch_one(pool)
                .await?;
            let items = sqlx::query_as::<_, Question>(include_str!("questions/list.sql"))
                .bind(project_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;
            (total, items)
        }
    };

    Ok(Page {
        items,
        total,
        limit,
        offset,
    })
}

/// Record an answer to a question.
///
/// 1. Stores the answer text (+ question for context) as a raw entity.
/// 2. Links it in the answers table.
/// 3. Marks the question as answered.
///
/// Returns the created Answer record.
pub async fn answer(
    pool: &SqlitePool,
    question: &Question,
    answer_text: &str,
) -> Result<Answer, AppError> {
    let mut tx = pool.begin().await?;

    // Store answer as a raw entity so it's part of the knowledge graph.
    let entity = entities::create_in_tx(
        &mut tx,
        question.project_id,
        EntityType::Raw,
        serde_json::json!({
            "question": question.question,
            "answer": answer_text,
        }),
        vec![],
    )
    .await?;

    let answer_id = Uuid::new_v4();
    sqlx::query(include_str!("questions/insert_answer.sql"))
        .bind(answer_id)
        .bind(question.id)
        .bind(entity.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(include_str!("questions/mark_answered.sql"))
        .bind(question.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Answer {
        id: answer_id,
        question_id: question.id,
        entity_id: entity.id,
        received_at: entity.created_at,
    })
}
