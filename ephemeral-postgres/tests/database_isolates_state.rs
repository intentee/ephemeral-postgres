use ephemeral_postgres::cluster::Cluster;
use sqlx::Row;

#[tokio::test]
async fn database_isolates_state_from_a_separate_database() {
    let cluster = Cluster::start().await.unwrap();
    let first = cluster.create_database().await.unwrap();
    let second = cluster.create_database().await.unwrap();

    sqlx::query("CREATE TABLE isolation_check (value INT)")
        .execute(first.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO isolation_check (value) VALUES (1)")
        .execute(first.pool())
        .await
        .unwrap();

    let stored_value: i32 = sqlx::query("SELECT value FROM isolation_check")
        .fetch_one(first.pool())
        .await
        .unwrap()
        .get("value");
    assert_eq!(stored_value, 1);

    let lookup_in_second_database = sqlx::query("SELECT value FROM isolation_check")
        .fetch_one(second.pool())
        .await;

    let error = lookup_in_second_database.expect_err(
        "table created in one ephemeral database must not be visible to a different database",
    );
    let sqlx::Error::Database(database_error) = error else {
        panic!("expected sqlx::Error::Database, got: {error:#?}");
    };
    assert_eq!(database_error.code().as_deref(), Some("42P01"));
}
