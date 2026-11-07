use sqlx::Row;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_connection_params::ClusterConnectionParams;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[tokio::test]
async fn connected_cluster_creates_databases_on_the_running_server() {
    let started = Cluster::start(ClusterParams::new(postgres_test_image()))
        .await
        .unwrap();
    let connected = Cluster::connect(ClusterConnectionParams::new(started.base_url()))
        .await
        .unwrap();
    let database = connected.create_database().await.unwrap();

    let value: i32 = sqlx::query("SELECT 1::int AS value")
        .fetch_one(database.pool())
        .await
        .unwrap()
        .get("value");

    assert_eq!(value, 1);
    assert_eq!(connected.base_url(), started.base_url());
}
