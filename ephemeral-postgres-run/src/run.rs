use tokio::process::Command;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;

use crate::constants::SERVER_URL_VARIABLE;
use crate::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use crate::run_arguments::RunArguments;

/// # Errors
///
/// Fails when the arguments are invalid, the cluster cannot start, or the command cannot run or
/// ends without an exit code.
pub async fn run(arguments: Vec<String>) -> Result<i32, EphemeralPostgresRunError> {
    let RunArguments {
        command,
        command_arguments,
        image,
    } = RunArguments::parse(arguments)?;
    let cluster = Cluster::start(ClusterParams::new(image))
        .await
        .map_err(|source| EphemeralPostgresRunError::ClusterStart { source })?;
    let status = Command::new(&command)
        .args(&command_arguments)
        .env(SERVER_URL_VARIABLE, cluster.base_url())
        .status()
        .await
        .map_err(|source| EphemeralPostgresRunError::CommandSpawn {
            command: command.clone(),
            source,
        })?;

    status
        .code()
        .ok_or(EphemeralPostgresRunError::CommandTerminated { command, status })
}
