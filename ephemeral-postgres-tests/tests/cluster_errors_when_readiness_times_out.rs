use std::time::Duration;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[tokio::test]
async fn errors_when_readiness_timeout_is_too_short() {
    let result = Cluster::start(ClusterParams {
        readiness_timeout: Duration::ZERO,
        ..ClusterParams::new(postgres_test_image())
    })
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::ReadinessTimeout { .. })
    ));
}
