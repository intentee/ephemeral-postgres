use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;
use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run::run;

#[tokio::test]
async fn reports_an_image_that_cannot_start_a_cluster() {
    let result = run(vec![
        "postgres".to_owned(),
        "tag-that-does-not-exist-9999".to_owned(),
        "--".to_owned(),
        "true".to_owned(),
    ])
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresRunError::ClusterStart {
            source: EphemeralPostgresError::ContainerStart { .. }
        })
    ));
}
