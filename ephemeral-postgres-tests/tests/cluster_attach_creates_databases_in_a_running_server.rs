use ephemeral_postgres::attach_params::AttachParams;
use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;
use sqlx::Row;

#[tokio::test]
async fn attached_cluster_creates_databases_in_the_running_server() {
    let running = Cluster::start(ClusterParams::new(postgres_test_image()))
        .await
        .unwrap();
    let attached = Cluster::attach(AttachParams::new(running.base_url()))
        .await
        .unwrap();
    let database = attached.create_database().await.unwrap();

    let result: i32 = sqlx::query("SELECT 1::int AS value")
        .fetch_one(database.pool())
        .await
        .unwrap()
        .get("value");

    assert_eq!(attached.base_url(), running.base_url());
    assert_eq!(result, 1);
}
