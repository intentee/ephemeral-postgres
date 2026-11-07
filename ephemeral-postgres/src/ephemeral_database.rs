use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;

use crate::database_lease::DatabaseLease;
use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::server_environment::ServerEnvironment;

pub struct EphemeralDatabase {
    connect_options: PgConnectOptions,
    pool: PgPool,
    _lease: DatabaseLease,
}

impl EphemeralDatabase {
    /// # Errors
    ///
    /// Fails when `EPHEMERAL_POSTGRES_URL` does not describe the server explicitly and
    /// unambiguously, or when the database cannot be created on that server.
    pub async fn create() -> Result<Self, EphemeralPostgresError> {
        Self::create_in(ServerEnvironment::current()).await
    }

    async fn create_in(environment: ServerEnvironment) -> Result<Self, EphemeralPostgresError> {
        let mut lease = DatabaseLease::acquire(environment.server_connect_options()?).await?;

        lease.create_with_pool().await.map(|pool| Self {
            connect_options: lease.database_options(),
            pool,
            _lease: lease,
        })
    }

    #[must_use]
    pub const fn connect_options(&self) -> &PgConnectOptions {
        &self.connect_options
    }

    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::io::ErrorKind;

    use super::EphemeralDatabase;
    use crate::database_exists::database_exists;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;
    use crate::server_environment::ServerEnvironment;

    #[tokio::test]
    async fn removes_its_database_when_dropped() {
        let database = EphemeralDatabase::create().await.unwrap();
        let database_name = database
            .connect_options()
            .get_database()
            .unwrap()
            .to_owned();

        drop(database);

        let observer = EphemeralDatabase::create().await.unwrap();

        assert!(!database_exists(observer.pool(), &database_name).await);
    }

    #[tokio::test]
    async fn rejects_an_environment_without_a_server_url() {
        assert!(matches!(
            EphemeralDatabase::create_in(ServerEnvironment {
                variables: BTreeMap::new(),
            })
            .await,
            Err(EphemeralPostgresError::MissingServerUrl { variable })
                if variable == "EPHEMERAL_POSTGRES_URL"
        ));
    }

    #[tokio::test]
    async fn rejects_a_server_it_cannot_reach() {
        assert!(matches!(
            EphemeralDatabase::create_in(ServerEnvironment {
                variables: BTreeMap::from([(
                    OsString::from("EPHEMERAL_POSTGRES_URL"),
                    OsString::from("postgres://postgres@127.0.0.1:1/postgres?sslmode=disable"),
                )]),
            })
            .await,
            Err(EphemeralPostgresError::AdminConnection { source: sqlx::Error::Io(io_error) })
                if io_error.kind() == ErrorKind::ConnectionRefused
        ));
    }
}
