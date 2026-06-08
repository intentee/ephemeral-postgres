use ephemeral_postgres::cluster::Cluster;
use uuid::Uuid;

#[tokio::test]
async fn create_with_id_errors_on_duplicate() {
    let cluster = Cluster::start().await.unwrap();
    let database_id = Uuid::new_v4();

    let first = cluster.create_database_with_id(database_id).await.unwrap();
    assert_eq!(first.db_name(), format!("test_{}", database_id.simple()));

    let result = cluster.create_database_with_id(database_id).await;

    assert!(
        result.is_err(),
        "create_database_with_id must fail when the requested database already exists",
    );
}
