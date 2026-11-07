use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use tokio::process::Command;

use crate::ephemeral_postgres_run_error::EphemeralPostgresRunError;
use crate::run_arguments::RunArguments;
use crate::server_url_variable::SERVER_URL_VARIABLE;

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
