use std::thread;

use sqlx::Connection;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;

use crate::database_identity::DatabaseIdentity;
use crate::database_registry::DatabaseRegistry;
use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::lease_stage::LeaseStage;

pub struct DatabaseLease {
    pub admin_options: PgConnectOptions,
    pub connection: PgConnection,
    pub identity: DatabaseIdentity,
    pub stage: LeaseStage,
}

impl DatabaseLease {
    pub async fn acquire(admin_options: PgConnectOptions) -> Result<Self, EphemeralPostgresError> {
        let mut connection = PgConnection::connect_with(
            &admin_options
                .clone()
                .options([("idle_session_timeout", "0")]),
        )
        .await
        .map_err(|source| EphemeralPostgresError::AdminConnection { source })?;
        let mut registry = DatabaseRegistry {
            connection: &mut connection,
        };

        registry.prepare().await?;
        registry.reclaim_orphans().await?;

        registry.reserve().await.map(|identity| Self {
            admin_options,
            connection,
            identity,
            stage: LeaseStage::Reserved,
        })
    }

    pub async fn create_with_pool(&mut self) -> Result<PgPool, EphemeralPostgresError> {
        self.identity.create(&mut self.connection).await?;
        self.stage = LeaseStage::Created;

        self.open_pool().await
    }

    #[must_use]
    pub fn database_options(&self) -> PgConnectOptions {
        self.admin_options.clone().database(&self.identity.name())
    }

    pub async fn open_pool(&self) -> Result<PgPool, EphemeralPostgresError> {
        PgPool::connect_with(self.database_options())
            .await
            .map_err(|source| EphemeralPostgresError::OpenPool {
                database_name: self.identity.name(),
                source,
            })
    }
}

impl Drop for DatabaseLease {
    fn drop(&mut self) {
        let Self {
            admin_options,
            identity,
            stage,
            ..
        } = self;

        thread::scope(|scope| {
            scope.spawn(|| identity.release_in_own_runtime(admin_options, stage));
        });
    }
}

#[cfg(test)]
mod tests {
    use sqlx::AssertSqlSafe;
    use sqlx::Connection;
    use sqlx::Error;
    use sqlx::PgConnection;
    use sqlx::PgPool;
    use sqlx::postgres::PgConnectOptions;
    use sqlx::query;
    use sqlx::raw_sql;

    use ephemeral_postgres_tests::block_on::block_on;
    use ephemeral_postgres_tests::database_exists::database_exists;
    use ephemeral_postgres_tests::has_sqlstate::has_sqlstate;
    use ephemeral_postgres_tests::held_advisory_lock::HeldAdvisoryLock;
    use ephemeral_postgres_tests::test_result::TestResult;
    use ephemeral_postgres_tests::verify::verify;

    use super::DatabaseLease;
    use crate::constants::FIRST_ASSIGNABLE_OID;
    use crate::database_identity::DatabaseIdentity;
    use crate::database_registry::DatabaseRegistry;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;
    use crate::server_environment::ServerEnvironment;

    async fn created_lease(
        admin_options: PgConnectOptions,
    ) -> Result<DatabaseLease, EphemeralPostgresError> {
        let mut lease = DatabaseLease::acquire(admin_options).await?;

        lease.create_with_pool().await?;

        Ok(lease)
    }

    async fn registry_host() -> Result<DatabaseLease, EphemeralPostgresError> {
        created_lease(ServerEnvironment::current().server_connect_options()?).await
    }

    async fn create_unregistered_database(pool: &PgPool, database_name: &str) -> Result<(), Error> {
        raw_sql(AssertSqlSafe(format!(
            "CREATE DATABASE \"{database_name}\" TEMPLATE template0"
        )))
        .execute(pool)
        .await?;

        Ok(())
    }

    #[test]
    fn rejects_a_registry_it_cannot_prepare() -> TestResult {
        block_on(async {
            let host = registry_host().await?;

            raw_sql(AssertSqlSafe(format!(
                "ALTER DATABASE \"{}\" SET default_transaction_read_only = on",
                host.identity.name()
            )))
            .execute(&host.open_pool().await?)
            .await?;

            verify(matches!(
                DatabaseLease::acquire(host.database_options()).await,
                Err(EphemeralPostgresError::PrepareRegistry { source })
                    if has_sqlstate(&source, "25006")
            ))?;

            Ok(())
        })
    }

    #[test]
    fn rejects_a_registry_it_cannot_read() -> TestResult {
        block_on(async {
            let host = registry_host().await?;

            raw_sql(
                "CREATE SCHEMA ephemeral_postgres; \
                 CREATE TABLE ephemeral_postgres.databases (unrelated integer)",
            )
            .execute(&host.open_pool().await?)
            .await?;

            verify(matches!(
                DatabaseLease::acquire(host.database_options()).await,
                Err(EphemeralPostgresError::FindOrphanedDatabases { source })
                    if has_sqlstate(&source, "42703")
            ))?;

            Ok(())
        })
    }

    #[test]
    fn rejects_a_registry_that_refuses_new_rows() -> TestResult {
        block_on(async {
            let host = registry_host().await?;

            raw_sql(
                "CREATE SCHEMA ephemeral_postgres; \
                 CREATE TABLE ephemeral_postgres.databases \
                 (database_oid integer PRIMARY KEY, blocker integer NOT NULL)",
            )
            .execute(&host.open_pool().await?)
            .await?;

            verify(matches!(
                DatabaseLease::acquire(host.database_options()).await,
                Err(EphemeralPostgresError::ReserveDatabase { source })
                    if has_sqlstate(&source, "23502")
            ))?;

            Ok(())
        })
    }

    #[test]
    fn rejects_a_pool_against_a_database_it_has_not_created() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let lease = DatabaseLease::acquire(host.database_options()).await?;

            verify(matches!(
                lease.open_pool().await,
                Err(EphemeralPostgresError::OpenPool { source, .. })
                    if has_sqlstate(&source, "3D000")
            ))?;

            Ok(())
        })
    }

    #[test]
    fn rejects_a_taken_name_and_leaves_the_database_that_holds_it() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let host_pool = host.open_pool().await?;
            let mut lease = DatabaseLease::acquire(host.database_options()).await?;
            let taken = DatabaseIdentity {
                oid: lease.identity.oid,
            };
            let database_name = taken.name();

            create_unregistered_database(&host_pool, &database_name).await?;

            verify(matches!(
                lease.create_with_pool().await,
                Err(EphemeralPostgresError::CreateDatabase { database_name: rejected, source })
                    if rejected == database_name && has_sqlstate(&source, "42P04")
            ))?;

            drop(lease);

            verify(database_exists(&host_pool, &database_name).await?)?;

            taken
                .drop_database(&mut *host_pool.acquire().await?)
                .await?;

            Ok(())
        })
    }

    #[test]
    fn reclaims_a_database_whose_owner_session_ended() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let host_pool = host.open_pool().await?;
            let mut lease = created_lease(host.database_options()).await?;
            let database_name = lease.identity.name();
            let lease_lock = HeldAdvisoryLock::held_by(&host_pool, &mut lease.connection).await?;
            let mut registry_connection =
                PgConnection::connect_with(&host.database_options()).await?;

            query("SELECT pg_terminate_backend($1)")
                .bind(lease_lock.backend_pid)
                .execute(&host_pool)
                .await?;
            lease_lock
                .wait_until_released(&mut registry_connection)
                .await?;
            DatabaseRegistry {
                connection: &mut registry_connection,
            }
            .reclaim_orphans()
            .await?;

            verify(!database_exists(&host_pool, &database_name).await?)?;

            Ok(())
        })
    }

    #[test]
    fn keeps_a_database_whose_oid_differs_from_the_registered_one() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let host_pool = host.open_pool().await?;
            let mut owner_connection = PgConnection::connect_with(&host.database_options()).await?;
            let mut owner_registry = DatabaseRegistry {
                connection: &mut owner_connection,
            };

            owner_registry.prepare().await?;

            let identity = owner_registry.reserve().await?;
            let database_name = identity.name();
            let owner_lock = HeldAdvisoryLock::held_by(&host_pool, &mut owner_connection).await?;
            let mut registry_connection =
                PgConnection::connect_with(&host.database_options()).await?;

            drop(owner_connection);
            owner_lock
                .wait_until_released(&mut registry_connection)
                .await?;
            create_unregistered_database(&host_pool, &database_name).await?;
            DatabaseRegistry {
                connection: &mut registry_connection,
            }
            .reclaim_orphans()
            .await?;

            verify(database_exists(&host_pool, &database_name).await?)?;

            identity.drop_database(&mut registry_connection).await?;

            Ok(())
        })
    }

    #[test]
    fn rejects_reclaiming_the_database_it_is_connected_to() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let mut registry_connection =
                PgConnection::connect_with(&host.database_options()).await?;
            let mut registry = DatabaseRegistry {
                connection: &mut registry_connection,
            };

            registry.prepare().await?;
            query("INSERT INTO ephemeral_postgres.databases (database_oid) VALUES ($1)")
                .bind(host.identity.oid)
                .execute(&mut *registry.connection)
                .await?;

            verify(matches!(
                registry.reclaim_orphans().await,
                Err(EphemeralPostgresError::DropOrphanedDatabase { database_name, source })
                    if database_name == host.identity.name() && has_sqlstate(&source, "55006")
            ))?;

            Ok(())
        })
    }

    #[test]
    fn rejects_an_orphan_it_cannot_unregister() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let mut registry_connection =
                PgConnection::connect_with(&host.database_options()).await?;
            let mut registry = DatabaseRegistry {
                connection: &mut registry_connection,
            };

            registry.prepare().await?;
            raw_sql(AssertSqlSafe(format!(
                "CREATE FUNCTION refuse_unregistering() RETURNS trigger LANGUAGE plpgsql \
                 AS $$ BEGIN RAISE EXCEPTION 'refused'; END $$; \
                 CREATE TRIGGER refuse_unregistering BEFORE DELETE ON ephemeral_postgres.databases \
                 FOR EACH ROW EXECUTE FUNCTION refuse_unregistering(); \
                 INSERT INTO ephemeral_postgres.databases (database_oid) VALUES ({FIRST_ASSIGNABLE_OID})"
            )))
            .execute(&mut *registry.connection)
            .await?;

            verify(matches!(
                registry.reclaim_orphans().await,
                Err(EphemeralPostgresError::UnregisterOrphanedDatabase { database_name, source })
                    if database_name == DatabaseIdentity { oid: FIRST_ASSIGNABLE_OID }.name()
                        && has_sqlstate(&source, "P0001")
            ))?;

            Ok(())
        })
    }

    #[test]
    fn leaves_its_database_to_the_next_reclaim_when_cleanup_cannot_connect() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let host_pool = host.open_pool().await?;
            let mut lease = created_lease(host.database_options()).await?;
            let database_name = lease.identity.name();
            let lease_lock = HeldAdvisoryLock::held_by(&host_pool, &mut lease.connection).await?;
            let mut registry_connection =
                PgConnection::connect_with(&host.database_options()).await?;

            lease.admin_options = host.database_options().port(1);
            drop(lease);
            lease_lock
                .wait_until_released(&mut registry_connection)
                .await?;

            verify(database_exists(&host_pool, &database_name).await?)?;

            DatabaseRegistry {
                connection: &mut registry_connection,
            }
            .reclaim_orphans()
            .await?;

            verify(!database_exists(&host_pool, &database_name).await?)?;

            Ok(())
        })
    }

    #[test]
    fn leaves_its_database_to_the_next_reclaim_when_cleanup_cannot_drop_it() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let host_pool = host.open_pool().await?;
            let mut lease = created_lease(host.database_options()).await?;
            let database_name = lease.identity.name();
            let lease_lock = HeldAdvisoryLock::held_by(&host_pool, &mut lease.connection).await?;
            let mut registry_connection =
                PgConnection::connect_with(&host.database_options()).await?;

            lease.admin_options = lease.database_options();
            drop(lease);
            lease_lock
                .wait_until_released(&mut registry_connection)
                .await?;

            verify(database_exists(&host_pool, &database_name).await?)?;

            DatabaseRegistry {
                connection: &mut registry_connection,
            }
            .reclaim_orphans()
            .await?;

            verify(!database_exists(&host_pool, &database_name).await?)?;

            Ok(())
        })
    }

    #[test]
    fn removes_its_database_even_when_it_cannot_unregister_it() -> TestResult {
        block_on(async {
            let host = registry_host().await?;
            let host_pool = host.open_pool().await?;
            let lease = created_lease(host.database_options()).await?;
            let database_name = lease.identity.name();

            query("DROP TABLE ephemeral_postgres.databases")
                .execute(&host_pool)
                .await?;
            drop(lease);

            verify(!database_exists(&host_pool, &database_name).await?)?;

            Ok(())
        })
    }
}
