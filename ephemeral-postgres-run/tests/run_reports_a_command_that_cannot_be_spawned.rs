use std::io::ErrorKind;

use ephemeral_postgres::postgres_image::PostgresImage;
use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run::run;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[tokio::test]
async fn reports_a_command_that_does_not_exist() {
    let PostgresImage { name, tag } = postgres_test_image();

    let result = run(vec![
        name,
        tag,
        "--".to_owned(),
        "/nonexistent/command".to_owned(),
    ])
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresRunError::CommandSpawn { command, source })
            if command == "/nonexistent/command" && source.kind() == ErrorKind::NotFound
    ));
}
