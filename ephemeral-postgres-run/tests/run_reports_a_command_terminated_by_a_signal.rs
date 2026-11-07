use std::os::unix::process::ExitStatusExt;

use ephemeral_postgres::postgres_image::PostgresImage;
use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run::run;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

const SIGKILL: i32 = 9;

#[tokio::test]
async fn reports_a_command_that_ends_without_an_exit_code() {
    let PostgresImage { name, tag } = postgres_test_image();

    let result = run(vec![
        name,
        tag,
        "--".to_owned(),
        "sh".to_owned(),
        "-c".to_owned(),
        "kill -KILL $$".to_owned(),
    ])
    .await;

    assert!(matches!(
        result,
        Err(EphemeralPostgresRunError::CommandTerminated { command, status })
            if command == "sh" && status.signal() == Some(SIGKILL)
    ));
}
