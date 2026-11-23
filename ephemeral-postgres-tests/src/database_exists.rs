use sqlx::PgPool;
use sqlx::query_scalar;

use crate::ephemeral_postgres_tests_error::EphemeralPostgresTestsError;

/// # Errors
///
/// Fails when the server cannot report whether the database exists.
pub async fn database_exists(
    pool: &PgPool,
    database_name: &str,
) -> Result<bool, EphemeralPostgresTestsError> {
    query_scalar("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
        .bind(database_name)
        .fetch_one(pool)
        .await
        .map_err(|source| EphemeralPostgresTestsError::CheckDatabaseExists {
            database_name: database_name.to_owned(),
            source,
        })
}
