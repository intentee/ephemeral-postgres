use std::process::Command;

use ephemeral_postgres::postgres_image::PostgresImage;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[test]
fn exits_with_a_failure_when_the_command_cannot_run() {
    let PostgresImage { name, tag } = postgres_test_image();

    let output = Command::new(env!("CARGO_BIN_EXE_ephemeral-postgres-run"))
        .args([name.as_str(), tag.as_str(), "--", "/nonexistent/command"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "ephemeral-postgres-run: failed to run `/nonexistent/command`\n  caused by: No such file or directory (os error 2)\n"
    );
}
