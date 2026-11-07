use std::env;
use std::error::Error;
use std::process;

use ephemeral_postgres_run::run::run;

fn report(error: &dyn Error) {
    eprintln!("ephemeral-postgres-run: {error}");

    let mut source = error.source();

    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match run(env::args().skip(1).collect()).await {
        Ok(code) => process::exit(code),
        Err(error) => {
            report(&error);
            process::exit(1);
        }
    }
}
