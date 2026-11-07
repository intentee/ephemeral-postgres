use sqlx::Row;
use sqlx::error::DatabaseError;
use sqlx::query;

use ephemeral_postgres::ephemeral_database::EphemeralDatabase;

#[tokio::test]
async fn isolates_state_from_another_ephemeral_database() {
    let first = EphemeralDatabase::create().await.unwrap();
    let second = EphemeralDatabase::create().await.unwrap();

    query("CREATE TABLE isolation_check (value INT)")
        .execute(first.pool())
        .await
        .unwrap();
    query("INSERT INTO isolation_check (value) VALUES (1)")
        .execute(first.pool())
        .await
        .unwrap();

    let stored_value: i32 = query("SELECT value FROM isolation_check")
        .fetch_one(first.pool())
        .await
        .unwrap()
        .get("value");
    assert_eq!(stored_value, 1);

    let error = query("SELECT value FROM isolation_check")
        .fetch_one(second.pool())
        .await
        .expect_err("a table created in one ephemeral database must not be visible in another");

    assert_eq!(
        error
            .as_database_error()
            .and_then(DatabaseError::code)
            .as_deref(),
        Some("42P01")
    );
}
