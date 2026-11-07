use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run_arguments::RunArguments;

#[test]
fn rejects_arguments_that_end_after_the_image() {
    assert!(matches!(
        RunArguments::parse(vec!["postgres".to_owned(), "18".to_owned()]),
        Err(EphemeralPostgresRunError::MissingSeparator)
    ));
}
