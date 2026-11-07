use std::thread;

use sqlx::Connection;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;

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
        PgPoolOptions::new()
            .max_connections(5)
            .connect_with(self.database_options())
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
    use sqlx::PgConnection;
    use sqlx::error::DatabaseError;
    use sqlx::query;
    use sqlx::query_scalar;
    use sqlx::raw_sql;

    use super::DatabaseLease;
    use crate::database_exists::database_exists;
    use crate::database_registry::DatabaseRegistry;
    use crate::ephemeral_postgres_error::EphemeralPostgresError;
    use crate::held_advisory_lock::HeldAdvisoryLock;
    use crate::server_environment::ServerEnvironment;

    async fn registry_host() -> DatabaseLease {
        let mut host = DatabaseLease::acquire(
            ServerEnvironment::current()
                .server_connect_options()
                .unwrap(),
        )
        .await
        .unwrap();

        host.create_with_pool().await.unwrap();

        host
    }

    #[tokio::test]
    async fn rejects_a_registry_it_cannot_prepare() {
        let host = registry_host().await;

        raw_sql(AssertSqlSafe(format!(
            "ALTER DATABASE \"{}\" SET default_transaction_read_only = on",
            host.identity.name()
        )))
        .execute(&host.open_pool().await.unwrap())
        .await
        .unwrap();

        assert!(matches!(
            DatabaseLease::acquire(host.database_options()).await,
            Err(EphemeralPostgresError::PrepareRegistry { source })
                if source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("25006")
        ));
    }

    #[tokio::test]
    async fn rejects_a_registry_it_cannot_read() {
        let host = registry_host().await;

        raw_sql(
            "CREATE SCHEMA ephemeral_postgres; \
             CREATE TABLE ephemeral_postgres.databases (unrelated integer)",
        )
        .execute(&host.open_pool().await.unwrap())
        .await
        .unwrap();

        assert!(matches!(
            DatabaseLease::acquire(host.database_options()).await,
            Err(EphemeralPostgresError::ReclaimOrphans { source })
                if source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("42703")
        ));
    }

    #[tokio::test]
    async fn rejects_a_registry_that_refuses_new_rows() {
        let host = registry_host().await;

        raw_sql(
            "CREATE SCHEMA ephemeral_postgres; \
             CREATE TABLE ephemeral_postgres.databases \
             (database_oid integer PRIMARY KEY, blocker integer NOT NULL)",
        )
        .execute(&host.open_pool().await.unwrap())
        .await
        .unwrap();

        assert!(matches!(
            DatabaseLease::acquire(host.database_options()).await,
            Err(EphemeralPostgresError::ReserveDatabase { source })
                if source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("23502")
        ));
    }

    #[tokio::test]
    async fn rejects_a_pool_against_a_database_it_has_not_created() {
        let host = registry_host().await;
        let lease = DatabaseLease::acquire(host.database_options())
            .await
            .unwrap();

        assert!(matches!(
            lease.open_pool().await,
            Err(EphemeralPostgresError::OpenPool { source, .. })
                if source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("3D000")
        ));
    }

    #[tokio::test]
    async fn rejects_a_taken_name_and_leaves_the_database_that_holds_it() {
        let host = registry_host().await;
        let host_pool = host.open_pool().await.unwrap();
        let mut lease = DatabaseLease::acquire(host.database_options())
            .await
            .unwrap();
        let database_name = lease.identity.name();

        raw_sql(AssertSqlSafe(format!(
            "CREATE DATABASE \"{database_name}\""
        )))
        .execute(&host_pool)
        .await
        .unwrap();

        assert!(matches!(
            lease.create_with_pool().await,
            Err(EphemeralPostgresError::CreateDatabase { database_name: rejected, source })
                if rejected == database_name
                    && source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("42P04")
        ));

        drop(lease);

        assert!(database_exists(&host_pool, &database_name).await);

        raw_sql(AssertSqlSafe(format!("DROP DATABASE \"{database_name}\"")))
            .execute(&host_pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reclaims_a_database_whose_owner_session_ended() {
        let host = registry_host().await;
        let host_pool = host.open_pool().await.unwrap();
        let mut lease = DatabaseLease::acquire(host.database_options())
            .await
            .unwrap();

        lease.create_with_pool().await.unwrap();

        let database_name = lease.identity.name();
        let lease_backend: i32 = query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut lease.connection)
            .await
            .unwrap();
        let lease_lock = HeldAdvisoryLock::held_by(&host_pool, lease_backend).await;
        let mut registry_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();

        query("SELECT pg_terminate_backend($1)")
            .bind(lease_backend)
            .execute(&host_pool)
            .await
            .unwrap();
        lease_lock
            .wait_until_released(&mut registry_connection)
            .await;
        DatabaseRegistry {
            connection: &mut registry_connection,
        }
        .reclaim_orphans()
        .await
        .unwrap();

        assert!(!database_exists(&host_pool, &database_name).await);
    }

    #[tokio::test]
    async fn keeps_a_database_whose_oid_differs_from_the_registered_one() {
        let host = registry_host().await;
        let host_pool = host.open_pool().await.unwrap();
        let mut owner_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();
        let owner_backend: i32 = query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut owner_connection)
            .await
            .unwrap();
        let mut owner_registry = DatabaseRegistry {
            connection: &mut owner_connection,
        };

        owner_registry.prepare().await.unwrap();

        let database_name = owner_registry.reserve().await.unwrap().name();
        let owner_lock = HeldAdvisoryLock::held_by(&host_pool, owner_backend).await;
        let mut registry_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();

        drop(owner_connection);
        owner_lock
            .wait_until_released(&mut registry_connection)
            .await;
        raw_sql(AssertSqlSafe(format!(
            "CREATE DATABASE \"{database_name}\""
        )))
        .execute(&host_pool)
        .await
        .unwrap();
        DatabaseRegistry {
            connection: &mut registry_connection,
        }
        .reclaim_orphans()
        .await
        .unwrap();

        assert!(database_exists(&host_pool, &database_name).await);

        raw_sql(AssertSqlSafe(format!("DROP DATABASE \"{database_name}\"")))
            .execute(&host_pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rejects_reclaiming_the_database_it_is_connected_to() {
        let host = registry_host().await;
        let mut registry_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();
        let mut registry = DatabaseRegistry {
            connection: &mut registry_connection,
        };

        registry.prepare().await.unwrap();
        query("INSERT INTO ephemeral_postgres.databases (database_oid) VALUES ($1)")
            .bind(host.identity.oid)
            .execute(&mut *registry.connection)
            .await
            .unwrap();

        assert!(matches!(
            registry.reclaim_orphans().await,
            Err(EphemeralPostgresError::ReclaimOrphans { source })
                if source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("55006")
        ));
    }

    #[tokio::test]
    async fn rejects_an_orphan_it_cannot_unregister() {
        let host = registry_host().await;
        let mut registry_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();
        let mut registry = DatabaseRegistry {
            connection: &mut registry_connection,
        };

        registry.prepare().await.unwrap();
        raw_sql(
            "CREATE FUNCTION refuse_unregistering() RETURNS trigger LANGUAGE plpgsql \
             AS $$ BEGIN RAISE EXCEPTION 'refused'; END $$; \
             CREATE TRIGGER refuse_unregistering BEFORE DELETE ON ephemeral_postgres.databases \
             FOR EACH ROW EXECUTE FUNCTION refuse_unregistering(); \
             INSERT INTO ephemeral_postgres.databases (database_oid) VALUES (16384)",
        )
        .execute(&mut *registry.connection)
        .await
        .unwrap();

        assert!(matches!(
            registry.reclaim_orphans().await,
            Err(EphemeralPostgresError::ReclaimOrphans { source })
                if source.as_database_error().and_then(DatabaseError::code).as_deref() == Some("P0001")
        ));
    }

    #[tokio::test]
    async fn leaves_its_database_to_the_next_reclaim_when_cleanup_cannot_connect() {
        let host = registry_host().await;
        let host_pool = host.open_pool().await.unwrap();
        let mut lease = DatabaseLease::acquire(host.database_options())
            .await
            .unwrap();

        lease.create_with_pool().await.unwrap();

        let database_name = lease.identity.name();
        let lease_backend: i32 = query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut lease.connection)
            .await
            .unwrap();
        let lease_lock = HeldAdvisoryLock::held_by(&host_pool, lease_backend).await;
        let mut registry_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();

        lease.admin_options = host.database_options().port(1);
        drop(lease);
        lease_lock
            .wait_until_released(&mut registry_connection)
            .await;

        assert!(database_exists(&host_pool, &database_name).await);

        DatabaseRegistry {
            connection: &mut registry_connection,
        }
        .reclaim_orphans()
        .await
        .unwrap();

        assert!(!database_exists(&host_pool, &database_name).await);
    }

    #[tokio::test]
    async fn leaves_its_database_to_the_next_reclaim_when_cleanup_cannot_drop_it() {
        let host = registry_host().await;
        let host_pool = host.open_pool().await.unwrap();
        let mut lease = DatabaseLease::acquire(host.database_options())
            .await
            .unwrap();

        lease.create_with_pool().await.unwrap();

        let database_name = lease.identity.name();
        let lease_backend: i32 = query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut lease.connection)
            .await
            .unwrap();
        let lease_lock = HeldAdvisoryLock::held_by(&host_pool, lease_backend).await;
        let mut registry_connection = PgConnection::connect_with(&host.database_options())
            .await
            .unwrap();

        lease.admin_options = lease.database_options();
        drop(lease);
        lease_lock
            .wait_until_released(&mut registry_connection)
            .await;

        assert!(database_exists(&host_pool, &database_name).await);

        DatabaseRegistry {
            connection: &mut registry_connection,
        }
        .reclaim_orphans()
        .await
        .unwrap();

        assert!(!database_exists(&host_pool, &database_name).await);
    }

    #[tokio::test]
    async fn removes_its_database_even_when_it_cannot_unregister_it() {
        let host = registry_host().await;
        let host_pool = host.open_pool().await.unwrap();
        let mut lease = DatabaseLease::acquire(host.database_options())
            .await
            .unwrap();

        lease.create_with_pool().await.unwrap();

        let database_name = lease.identity.name();

        query("DROP TABLE ephemeral_postgres.databases")
            .execute(&host_pool)
            .await
            .unwrap();
        drop(lease);

        assert!(!database_exists(&host_pool, &database_name).await);
    }
}
