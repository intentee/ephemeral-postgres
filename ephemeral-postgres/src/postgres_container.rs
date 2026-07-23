use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ContainerAsync;

use crate::ephemeral_postgres_error::EphemeralPostgresError;

const POSTGRES_CONTAINER_PORT: u16 = 5432;

#[doc(hidden)]
pub struct PostgresContainer {
    container: ContainerAsync<Postgres>,
}

impl PostgresContainer {
    #[must_use]
    pub fn new(container: ContainerAsync<Postgres>) -> Self {
        Self { container }
    }

    pub async fn mapped_host_port(&self) -> Result<u16, EphemeralPostgresError> {
        self.container
            .get_host_port_ipv4(POSTGRES_CONTAINER_PORT)
            .await
            .map_err(|source| EphemeralPostgresError::MappedPort {
                container_port: POSTGRES_CONTAINER_PORT,
                source,
            })
    }
}
