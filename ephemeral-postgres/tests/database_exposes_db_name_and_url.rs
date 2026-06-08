use ephemeral_postgres::cluster::Cluster;

#[tokio::test]
async fn db_name_is_unique_hex_and_database_url_extends_cluster_base_url() {
    let cluster = Cluster::start().await.unwrap();
    let database = cluster.create_database().await.unwrap();

    let db_name = database.db_name();
    assert!(db_name.starts_with("test_"));
    assert!(db_name.len() > "test_".len());

    let expected_url = format!("{}/{}", cluster.base_url(), db_name);
    assert_eq!(database.database_url(), expected_url);
}
