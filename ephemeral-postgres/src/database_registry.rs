use sqlx::AssertSqlSafe;
use sqlx::PgConnection;
use sqlx::query;
use sqlx::query_as;
use sqlx::query_scalar;
use sqlx::raw_sql;

use crate::database_identity::DatabaseIdentity;
use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::orphaned_database::OrphanedDatabase;

const FIRST_ASSIGNABLE_OID: i32 = 16384;
const LOCK_NAMESPACE: i32 = i32::from_be_bytes(*b"epgr");
const SETUP_LOCK: i32 = 0;

pub struct DatabaseRegistry<'connection> {
    pub connection: &'connection mut PgConnection,
}

impl DatabaseRegistry<'_> {
    pub async fn prepare(&mut self) -> Result<(), EphemeralPostgresError> {
        raw_sql(AssertSqlSafe(format!(
            "SELECT pg_advisory_xact_lock({LOCK_NAMESPACE}, {SETUP_LOCK}); \
             CREATE SCHEMA IF NOT EXISTS ephemeral_postgres; \
             CREATE TABLE IF NOT EXISTS ephemeral_postgres.databases (database_oid integer PRIMARY KEY)"
        )))
        .execute(&mut *self.connection)
        .await
        .map_err(|source| EphemeralPostgresError::PrepareRegistry { source })?;

        Ok(())
    }

    pub async fn reclaim_orphans(&mut self) -> Result<(), EphemeralPostgresError> {
        let orphans: Vec<OrphanedDatabase> = query_as(
            "WITH abandoned AS MATERIALIZED \
             (SELECT database_oid FROM ephemeral_postgres.databases \
              WHERE pg_try_advisory_lock($1, database_oid)) \
             SELECT abandoned.database_oid, catalog.datname AS database_name \
             FROM abandoned \
             LEFT JOIN pg_database AS catalog ON catalog.oid = abandoned.database_oid::oid",
        )
        .bind(LOCK_NAMESPACE)
        .fetch_all(&mut *self.connection)
        .await
        .map_err(|source| EphemeralPostgresError::ReclaimOrphans { source })?;

        for OrphanedDatabase {
            database_name,
            database_oid,
        } in orphans
        {
            let identity = DatabaseIdentity { oid: database_oid };

            if database_name == Some(identity.name()) {
                identity
                    .drop_database(self.connection)
                    .await
                    .map_err(|source| EphemeralPostgresError::ReclaimOrphans { source })?;
            }

            query(
                "WITH unregistered AS \
                 (DELETE FROM ephemeral_postgres.databases WHERE database_oid = $2 RETURNING database_oid) \
                 SELECT pg_advisory_unlock($1, database_oid) FROM unregistered",
            )
            .bind(LOCK_NAMESPACE)
            .bind(database_oid)
            .execute(&mut *self.connection)
            .await
            .map_err(|source| EphemeralPostgresError::ReclaimOrphans { source })?;
        }

        Ok(())
    }

    pub async fn reserve(&mut self) -> Result<DatabaseIdentity, EphemeralPostgresError> {
        query_scalar(
            "WITH reserved AS \
             (INSERT INTO ephemeral_postgres.databases (database_oid) \
              VALUES (($1::float8 + floor(random() * ($2::float8 - $1::float8 + 1)))::integer) \
              RETURNING database_oid) \
             SELECT database_oid FROM reserved WHERE pg_try_advisory_lock($3, database_oid)",
        )
        .bind(FIRST_ASSIGNABLE_OID)
        .bind(i32::MAX)
        .bind(LOCK_NAMESPACE)
        .fetch_one(&mut *self.connection)
        .await
        .map(|oid| DatabaseIdentity { oid })
        .map_err(|source| EphemeralPostgresError::ReserveDatabase { source })
    }
}
