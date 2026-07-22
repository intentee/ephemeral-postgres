use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;
use uuid::Uuid;

#[tokio::test]
async fn create_with_id_errors_on_duplicate() {
    let cluster = Cluster::start(ClusterParams::new(postgres_test_image()))
        .await
        .unwrap();
    let database_id = Uuid::new_v4();

    let first = cluster.create_database_with_id(database_id).await.unwrap();
    assert_eq!(first.db_name(), format!("test_{}", database_id.simple()));

    let result = cluster.create_database_with_id(database_id).await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::CreateDatabase { .. })
    ));
}
