use sqlx::FromRow;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::query;
use sqlx::query_as;
use sqlx::query_scalar;

use crate::ephemeral_postgres_tests_error::EphemeralPostgresTestsError;

#[derive(FromRow)]
pub struct HeldAdvisoryLock {
    pub backend_pid: i32,
    pub class_id: i32,
    pub object_id: i32,
}

impl HeldAdvisoryLock {
    /// # Errors
    ///
    /// Fails when the backend of the holder cannot be identified, or holds no advisory lock on a
    /// pair of keys.
    pub async fn held_by(
        pool: &PgPool,
        holder: &mut PgConnection,
    ) -> Result<Self, EphemeralPostgresTestsError> {
        let backend_pid: i32 = query_scalar("SELECT pg_backend_pid()")
            .fetch_one(holder)
            .await
            .map_err(|source| EphemeralPostgresTestsError::FindBackendPid { source })?;

        query_as(
            "SELECT pid AS backend_pid, classid::int4 AS class_id, objid::int4 AS object_id \
             FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 2 AND pid = $1",
        )
        .bind(backend_pid)
        .fetch_one(pool)
        .await
        .map_err(|source| EphemeralPostgresTestsError::FindHeldAdvisoryLock {
            backend_pid,
            source,
        })
    }

    /// # Errors
    ///
    /// Fails when the server cannot acquire or release the lock.
    pub async fn wait_until_released(
        &self,
        connection: &mut PgConnection,
    ) -> Result<(), EphemeralPostgresTestsError> {
        let Self {
            class_id,
            object_id,
            ..
        } = *self;

        query("SELECT pg_advisory_lock($1, $2)")
            .bind(class_id)
            .bind(object_id)
            .execute(&mut *connection)
            .await
            .map_err(|source| EphemeralPostgresTestsError::AcquireAdvisoryLock {
                class_id,
                object_id,
                source,
            })?;
        query("SELECT pg_advisory_unlock($1, $2)")
            .bind(class_id)
            .bind(object_id)
            .execute(connection)
            .await
            .map_err(|source| EphemeralPostgresTestsError::ReleaseAdvisoryLock {
                class_id,
                object_id,
                source,
            })?;

        Ok(())
    }
}
