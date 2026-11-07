use ephemeral_postgres::postgres_image::PostgresImage;
use ephemeral_postgres_run::run::run;
use ephemeral_postgres_tests::postgres_test_image::postgres_test_image;

#[tokio::test]
async fn returns_the_exit_code_of_a_command_that_sees_the_server_url() {
    let PostgresImage { name, tag } = postgres_test_image();

    let code = run(vec![
        name,
        tag,
        "--".to_owned(),
        "sh".to_owned(),
        "-c".to_owned(),
        "case \"$EPHEMERAL_POSTGRES_URL\" in postgres://postgres@127.0.0.1:*) exit 7;; esac; exit 1"
            .to_owned(),
    ])
    .await
    .unwrap();

    assert_eq!(code, 7);
}
