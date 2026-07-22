use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

#[tokio::test]
async fn finalize_errors_when_mapped_port_unavailable() {
    let ClusterParams {
        image,
        readiness_timeout,
    } = ClusterParams::new(postgres_test_image());

    let container = image
        .into_container_request()
        .start()
        .await
        .expect("postgres container should start");
    container
        .stop_with_timeout(Some(0))
        .await
        .expect("postgres container should stop before finalizing the cluster");

    let result = Cluster::from_started_container(container, readiness_timeout).await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::MappedPort { .. })
    ));
}
