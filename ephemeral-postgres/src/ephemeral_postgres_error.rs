use std::time::Duration;

use testcontainers_modules::testcontainers::TestcontainersError;

#[derive(Debug, thiserror::Error)]
pub enum EphemeralPostgresError {
    #[error("failed to start ephemeral postgres container")]
    ContainerStart {
        #[source]
        source: TestcontainersError,
    },

    #[error("failed to resolve ephemeral postgres mapped port {container_port}")]
    MappedPort {
        container_port: u16,
        #[source]
        source: TestcontainersError,
    },

    #[error("postgres admin pool did not become ready within {timeout:?}")]
    ReadinessTimeout { timeout: Duration },

    #[error("failed to create ephemeral database {db_name}")]
    CreateDatabase {
        db_name: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to open pool against ephemeral database {db_name}")]
    OpenPool {
        db_name: String,
        #[source]
        source: sqlx::Error,
    },
}
