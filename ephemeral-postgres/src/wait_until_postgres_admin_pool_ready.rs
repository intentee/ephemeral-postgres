use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::time::Instant;

const ADMIN_POOL_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub async fn wait_until_postgres_admin_pool_ready(
    admin_url: &str,
    timeout: Duration,
) -> Result<PgPool> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(anyhow!(
                "postgres admin pool did not become ready within {timeout:?}",
            ));
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

    use super::wait_until_postgres_admin_pool_ready;

    // A zero budget makes the deadline already elapsed, so the wait returns before its first
    // connection attempt.
    #[tokio::test]
    async fn errors_immediately_when_timeout_is_zero() {
        let result = wait_until_postgres_admin_pool_ready(
            "postgres://postgres@127.0.0.1:1/postgres",
            Duration::ZERO,
        )
        .await;

        assert!(result.is_err());
    }

    // Port 1 on loopback refuses connections immediately, so each poll fails fast and the deadline
    // ends the loop. The 200ms budget exceeds the 50ms poll interval, exercising the retry path
    // before the timeout error.
    #[tokio::test]
    async fn errors_after_polling_when_endpoint_unreachable() {
        let result = wait_until_postgres_admin_pool_ready(
            "postgres://postgres@127.0.0.1:1/postgres",
            Duration::from_millis(200),
        )
        .await;

        assert!(result.is_err());
    }
}
