use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::time::Instant;

use crate::ephemeral_postgres_error::EphemeralPostgresError;

const ADMIN_POOL_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub async fn wait_until_postgres_admin_pool_ready(
    admin_url: &str,
    timeout: Duration,
) -> Result<PgPool, EphemeralPostgresError> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(EphemeralPostgresError::ReadinessTimeout { timeout });
        }

        let attempt = tokio::time::timeout(
            remaining,
            PgPoolOptions::new().max_connections(5).connect(admin_url),
        )
        .await;
        if let Ok(Ok(pool)) = attempt {
            return Ok(pool);
        }

        let sleep_for =
            ADMIN_POOL_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now()));
        tokio::time::sleep(sleep_for).await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ADMIN_POOL_POLL_INTERVAL;
    use super::wait_until_postgres_admin_pool_ready;

    const UNREACHABLE_ADMIN_URL: &str = "postgres://postgres@127.0.0.1:1/postgres";

    #[tokio::test]
    async fn errors_immediately_when_timeout_is_zero() {
        let result =
            wait_until_postgres_admin_pool_ready(UNREACHABLE_ADMIN_URL, Duration::ZERO).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn errors_after_polling_when_endpoint_unreachable() {
        let budget_spanning_several_polls = ADMIN_POOL_POLL_INTERVAL * 4;

        let result = wait_until_postgres_admin_pool_ready(
            UNREACHABLE_ADMIN_URL,
            budget_spanning_several_polls,
        )
        .await;

        assert!(result.is_err());
    }
}
