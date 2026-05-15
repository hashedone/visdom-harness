use tempfile::NamedTempFile;
use visdom_harness::{db, projects};

fn tempfile_db_url(f: &NamedTempFile) -> String {
    format!("sqlite://{}?mode=rwc", f.path().display())
}

#[tokio::test]
async fn project_create_and_get_round_trip() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let created = projects::create(&pool, "test project", "a description")
        .await
        .unwrap();

    assert!(!created.id.is_empty());
    assert_eq!(created.name, "test project");
    assert!(!created.description_entity_id.is_empty());
    assert!(!created.created_at.is_empty());

    let fetched = projects::get(&pool, &created.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, created.name);
    assert_eq!(fetched.description_entity_id, created.description_entity_id);
    assert_eq!(fetched.created_at, created.created_at);
}

#[tokio::test]
async fn project_get_unknown_returns_none() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let result = projects::get(&pool, "00000000-0000-0000-0000-000000000000")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn project_exists_true_and_false() {
    let db_file = NamedTempFile::new().unwrap();
    let pool = db::connect_and_migrate(&tempfile_db_url(&db_file))
        .await
        .unwrap();

    let created = projects::create(&pool, "exists test", "desc")
        .await
        .unwrap();

    assert!(projects::exists(&pool, &created.id).await.unwrap());
    assert!(
        !projects::exists(&pool, "00000000-0000-0000-0000-000000000000")
            .await
            .unwrap()
    );
}
