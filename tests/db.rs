use sqlx::Row;
use tempfile::NamedTempFile;
use visdom_harness::db;

fn tempfile_db_url(f: &NamedTempFile) -> String {
    format!("sqlite://{}?mode=rwc", f.path().display())
}

#[tokio::test]
async fn migrations_create_expected_tables() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);

    let pool = db::connect_and_migrate(&db_url).await.unwrap();

    // _sqlx_migrations table exists
    let row = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .expect("_sqlx_migrations table not found");
    assert_eq!(row.get::<String, _>("name"), "_sqlx_migrations");

    // _meta table exists (from our migration)
    let row = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='_meta'",
    )
    .fetch_one(&pool)
    .await
    .expect("_meta table not found");
    assert_eq!(row.get::<String, _>("name"), "_meta");
}

#[tokio::test]
async fn migrations_are_idempotent() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);

    // First run
    let pool1 = db::connect_and_migrate(&db_url).await.unwrap();
    pool1.close().await;

    // Second run against the same file — must not error
    let pool2 = db::connect_and_migrate(&db_url).await.unwrap();
    pool2.close().await;
}
