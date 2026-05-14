use sqlx::Row;
use tempfile::NamedTempFile;
use visdom_harness::db;

fn tempfile_db_url(f: &NamedTempFile) -> String {
    format!("sqlite://{}?mode=rwc", f.path().display())
}

#[tokio::test]
async fn migrations_runner_wires_sqlx_tracking_table() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);

    let pool = db::connect_and_migrate(&db_url).await.unwrap();

    let row = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .expect("_sqlx_migrations table not found");
    assert_eq!(row.get::<String, _>("name"), "_sqlx_migrations");
}

#[tokio::test]
async fn migrations_are_idempotent() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);

    let pool1 = db::connect_and_migrate(&db_url).await.unwrap();
    pool1.close().await;

    let pool2 = db::connect_and_migrate(&db_url).await.unwrap();
    pool2.close().await;
}
