use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;
use sqlx::Row;

#[tokio::test]
async fn database_provides_pool_that_executes_queries() {
    let cluster = Cluster::start(ClusterParams::new(postgres_test_image()))
        .await
        .unwrap();
    let database = cluster.create_database().await.unwrap();

    let result: i32 = sqlx::query("SELECT 1::int AS value")
        .fetch_one(database.pool())
        .await
        .unwrap()
        .get("value");

    assert_eq!(result, 1);
}
