use sqlx::query_scalar;

use ephemeral_postgres::ephemeral_database::EphemeralDatabase;

#[tokio::test]
async fn provides_a_pool_connected_to_its_own_database() {
    let database = EphemeralDatabase::create().await.unwrap();

    let current_database: String = query_scalar("SELECT current_database()")
        .fetch_one(database.pool())
        .await
        .unwrap();

    assert_eq!(
        database.connect_options().get_database(),
        Some(current_database.as_str())
    );
}
