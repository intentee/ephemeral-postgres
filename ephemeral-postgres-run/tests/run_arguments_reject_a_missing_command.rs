use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run_arguments::RunArguments;

#[test]
fn rejects_a_separator_without_a_command() {
    assert!(matches!(
        RunArguments::parse(vec![
            "postgres".to_owned(),
            "18".to_owned(),
            "--".to_owned(),
        ]),
        Err(EphemeralPostgresRunError::MissingCommand)
    ));
}
