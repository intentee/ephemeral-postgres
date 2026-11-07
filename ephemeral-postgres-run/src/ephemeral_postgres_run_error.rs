use std::process::ExitStatus;

use ephemeral_postgres::ephemeral_postgres_error::EphemeralPostgresError;

#[derive(Debug, thiserror::Error)]
pub enum EphemeralPostgresRunError {
    #[error("failed to start the shared postgres cluster")]
    ClusterStart {
        #[source]
        source: EphemeralPostgresError,
    },

    #[error("failed to run `{command}`")]
    CommandSpawn {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("`{command}` ended without an exit code ({status})")]
    CommandTerminated { command: String, status: ExitStatus },

    #[error("missing the command to run after `--`")]
    MissingCommand,

    #[error(
        "missing the postgres image name; usage: ephemeral-postgres-run <image-name> <image-tag> -- <command> [arguments...]"
    )]
    MissingImageName,

    #[error(
        "missing the postgres image tag; usage: ephemeral-postgres-run <image-name> <image-tag> -- <command> [arguments...]"
    )]
    MissingImageTag,

    #[error("missing `--` between the postgres image and the command")]
    MissingSeparator,

    #[error("expected `--` after the postgres image, found `{found}`")]
    UnexpectedArgument { found: String },
}
