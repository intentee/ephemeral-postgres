use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;
use ephemeral_postgres::postgres_image::PostgresImage;

#[tokio::test]
async fn errors_when_image_tag_cannot_be_resolved() {
    let result = Cluster::start(ClusterParams::new(PostgresImage::new(
        "postgres",
        "tag-that-does-not-exist-9999",
    )))
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::ContainerStart { .. })
    ));
}
