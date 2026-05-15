use serde_json::json;
use tempfile::NamedTempFile;
use visdom_harness::{
    db,
    entities::{self, EntityType},
    projects,
};

fn tempfile_db_url(f: &NamedTempFile) -> String {
    format!("sqlite://{}?mode=rwc", f.path().display())
}

#[tokio::test]
async fn entity_create_and_get_each_type() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let project = projects::create(&pool, "entity test project", "desc")
        .await
        .unwrap();

    let raw = entities::create(
        &pool,
        &project.id,
        EntityType::Raw,
        json!({"source": "raw text"}),
        vec![],
    )
    .await
    .unwrap();

    let knowledge = entities::create(
        &pool,
        &project.id,
        EntityType::Knowledge,
        json!({"fact": "x"}),
        vec![],
    )
    .await
    .unwrap();

    let summary = entities::create(
        &pool,
        &project.id,
        EntityType::Summary,
        json!({"summary": "y"}),
        vec![raw.id.clone(), knowledge.id.clone()],
    )
    .await
    .unwrap();

    // round-trip raw
    let fetched_raw = entities::get(&pool, &raw.id).await.unwrap().unwrap();
    assert_eq!(fetched_raw.entity_type, EntityType::Raw);
    assert_eq!(fetched_raw.content, json!({"source": "raw text"}));
    assert!(fetched_raw.contributing_entity_ids.is_empty());

    // round-trip knowledge
    let fetched_knowledge = entities::get(&pool, &knowledge.id).await.unwrap().unwrap();
    assert_eq!(fetched_knowledge.entity_type, EntityType::Knowledge);
    assert_eq!(fetched_knowledge.content, json!({"fact": "x"}));
    assert!(fetched_knowledge.contributing_entity_ids.is_empty());

    // round-trip summary
    let fetched_summary = entities::get(&pool, &summary.id).await.unwrap().unwrap();
    assert_eq!(fetched_summary.entity_type, EntityType::Summary);
    assert_eq!(fetched_summary.content, json!({"summary": "y"}));
    assert_eq!(
        fetched_summary.contributing_entity_ids,
        vec![raw.id.clone(), knowledge.id.clone()]
    );
}

#[tokio::test]
async fn entity_get_unknown_returns_none() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let result = entities::get(&pool, "00000000-0000-0000-0000-000000000000")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn entity_list_by_project_returns_most_recent_first() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let project = projects::create(&pool, "list test", "desc").await.unwrap();

    let raw = entities::create(
        &pool,
        &project.id,
        EntityType::Raw,
        json!({"source": "raw text"}),
        vec![],
    )
    .await
    .unwrap();
    let knowledge = entities::create(
        &pool,
        &project.id,
        EntityType::Knowledge,
        json!({"fact": "x"}),
        vec![],
    )
    .await
    .unwrap();
    let summary = entities::create(
        &pool,
        &project.id,
        EntityType::Summary,
        json!({"summary": "y"}),
        vec![raw.id.clone(), knowledge.id.clone()],
    )
    .await
    .unwrap();

    let list = entities::list_by_project(&pool, &project.id, 10)
        .await
        .unwrap();
    // 1 description entity from project creation + 3 explicit entities
    assert_eq!(list.len(), 4);

    // all three IDs present regardless of sub-second ordering
    let ids: Vec<&str> = list.iter().map(|e| e.id.as_str()).collect();
    assert!(ids.contains(&raw.id.as_str()));
    assert!(ids.contains(&knowledge.id.as_str()));
    assert!(ids.contains(&summary.id.as_str()));
}

#[tokio::test]
async fn entity_insert_unknown_project_violates_foreign_key() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let result = entities::create(
        &pool,
        "00000000-0000-0000-0000-000000000000",
        EntityType::Raw,
        json!({"source": "orphan"}),
        vec![],
    )
    .await;

    assert!(result.is_err(), "expected FK violation but got Ok");
    // ensure the error is a DB-level foreign key constraint
    match result.unwrap_err() {
        visdom_harness::error::AppError::Db(sqlx::Error::Database(e)) => {
            let msg = e.message().to_lowercase();
            assert!(
                msg.contains("foreign key") || msg.contains("constraint"),
                "unexpected db error: {msg}"
            );
        }
        other => panic!("expected AppError::Db(sqlx::Error::Database), got: {other:?}"),
    }
}
