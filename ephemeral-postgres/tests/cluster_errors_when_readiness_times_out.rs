use std::time::Duration;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;

#[tokio::test]
async fn errors_when_readiness_timeout_is_too_short() {
    let result = Cluster::start_with_params(ClusterParams {
        readiness_timeout: Duration::ZERO,
        ..ClusterParams::default()
    })
    .await;

    assert!(
        result.is_err(),
        "start must error when the readiness timeout elapses before postgres accepts connections",
    );
}
