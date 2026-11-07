use std::time::Duration;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_connection_params::ClusterConnectionParams;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;

#[tokio::test]
async fn connect_errors_when_the_server_never_becomes_ready() {
    let result = Cluster::connect(ClusterConnectionParams {
        readiness_timeout: Duration::ZERO,
        ..ClusterConnectionParams::new("postgres://postgres@127.0.0.1:1")
    })
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::ReadinessTimeout { .. })
    ));
}
