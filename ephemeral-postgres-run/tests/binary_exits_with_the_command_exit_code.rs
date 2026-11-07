use std::process::Command;

use ephemeral_postgres::postgres_image::PostgresImage;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[test]
fn exits_with_the_exit_code_of_the_command() {
    let PostgresImage { name, tag } = postgres_test_image();

    let status = Command::new(env!("CARGO_BIN_EXE_ephemeral-postgres-run"))
        .args([name.as_str(), tag.as_str(), "--", "sh", "-c", "exit 5"])
        .status()
        .unwrap();

    assert_eq!(status.code(), Some(5));
}
