use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run_arguments::RunArguments;

#[test]
fn rejects_arguments_without_an_image_name() {
    assert!(matches!(
        RunArguments::parse(Vec::new()),
        Err(EphemeralPostgresRunError::MissingImageName)
    ));
}
