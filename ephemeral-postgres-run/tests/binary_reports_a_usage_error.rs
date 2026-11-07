use std::process::Command;

#[test]
fn exits_with_a_failure_and_the_usage_when_arguments_are_missing() {
    let output = Command::new(env!("CARGO_BIN_EXE_ephemeral-postgres-run"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "ephemeral-postgres-run: missing the postgres image name; usage: ephemeral-postgres-run <image-name> <image-tag> -- <command> [arguments...]\n"
    );
}
