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

    use sqlx::Error;

    use ephemeral_postgres_tests::block_on::block_on;
    use ephemeral_postgres_tests::database_exists::database_exists;
    use ephemeral_postgres_tests::test_result::TestResult;
    use ephemeral_postgres_tests::verify::verify;

    use super::EphemeralDatabase;
    use crate::constants::SERVER_URL_VARIABLE;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;
    use crate::server_environment::ServerEnvironment;

    #[test]
    fn removes_its_database_when_dropped() -> TestResult {
        block_on(async {
            let database = EphemeralDatabase::create().await?;
            let database_name = database.connect_options().get_database().map(str::to_owned);

            drop(database);

            let observer = EphemeralDatabase::create().await?;

            verify(match database_name {
                Some(name) => !database_exists(observer.pool(), &name).await?,
                None => false,
            })?;

            Ok(())
        })
    }

    #[test]
    fn rejects_an_environment_without_a_server_url() -> TestResult {
        block_on(async {
            verify(matches!(
                EphemeralDatabase::create_in(ServerEnvironment {
                    variables: BTreeMap::new(),
                })
                .await,
                Err(EphemeralPostgresError::MissingServerUrl { variable })
                    if variable == SERVER_URL_VARIABLE
            ))?;

            Ok(())
        })
    }

    #[test]
    fn rejects_a_server_it_cannot_reach() -> TestResult {
        block_on(async {
            verify(matches!(
                EphemeralDatabase::create_in(ServerEnvironment {
                    variables: BTreeMap::from([(
                        OsString::from(SERVER_URL_VARIABLE),
                        OsString::from("postgres://postgres@127.0.0.1:1/postgres?sslmode=disable"),
                    )]),
                })
                .await,
                Err(EphemeralPostgresError::AdminConnection { source: Error::Io(io_error) })
                    if io_error.kind() == ErrorKind::ConnectionRefused
            ))?;

            Ok(())
        })
    }
}
