use ephemeral_postgres_run::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use ephemeral_postgres_run::run_arguments::RunArguments;

#[test]
fn rejects_a_command_that_is_not_preceded_by_the_separator() {
    assert!(matches!(
        RunArguments::parse(vec![
            "postgres".to_owned(),
            "18".to_owned(),
            "true".to_owned(),
        ]),
        Err(EphemeralPostgresRunError::UnexpectedArgument { found }) if found == "true"
    ));
}
