use std::time::Duration;

use ephemeral_postgres::attach_params::AttachParams;
use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;

#[tokio::test]
async fn attach_errors_when_the_server_does_not_become_ready() {
    let result = Cluster::attach(AttachParams {
        readiness_timeout: Duration::ZERO,
        ..AttachParams::new("postgres://postgres@127.0.0.1:1")
    })
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresError::ReadinessTimeout { .. })
    ));
}
