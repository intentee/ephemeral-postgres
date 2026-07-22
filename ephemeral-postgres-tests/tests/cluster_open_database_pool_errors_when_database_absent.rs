use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[tokio::test]
async fn open_database_pool_errors_when_database_does_not_exist() {
    let cluster = Cluster::start(ClusterParams::new(postgres_test_image()))
        .await
        .unwrap();

    let result = cluster.open_database_pool("test_absent".to_owned()).await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::OpenPool { .. })
    ));
}
