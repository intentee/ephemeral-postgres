use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::time::Instant;

use crate::ephemeral_postgres_error::EphemeralPostgresError;

const ADMIN_POOL_POLL_INTERVAL: Duration = Duration::from_millis(50);

enum LastConnectionOutcome {
    NoConnectionErrorYet,
    Failed(sqlx::Error),
}

pub async fn wait_until_postgres_admin_pool_ready(
    admin_url: &str,
    timeout: Duration,
) -> Result<PgPool, EphemeralPostgresError> {
    let deadline = Instant::now() + timeout;
    let mut last_connection_outcome = LastConnectionOutcome::NoConnectionErrorYet;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(match last_connection_outcome {
                LastConnectionOutcome::NoConnectionErrorYet => {
                    EphemeralPostgresError::ReadinessTimeout { timeout }
                }
                LastConnectionOutcome::Failed(source) => {
                    EphemeralPostgresError::ReadinessTimeoutAfterConnectionError { timeout, source }
                }
            });
        }

        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(remaining)
            .connect(admin_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(connection_error) => {
                last_connection_outcome = LastConnectionOutcome::Failed(connection_error);
            }
        }

        let sleep_for =
            ADMIN_POOL_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now()));
        tokio::time::sleep(sleep_for).await;
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::time::Duration;

    use super::ADMIN_POOL_POLL_INTERVAL;
    use super::wait_until_postgres_admin_pool_ready;

    const UNREACHABLE_ADMIN_URL: &str = "postgres://postgres@127.0.0.1:1/postgres";

    #[tokio::test]
    async fn times_out_without_a_cause_when_no_connection_is_attempted() {
        let result =
            wait_until_postgres_admin_pool_ready(UNREACHABLE_ADMIN_URL, Duration::ZERO).await;

        assert!(result.unwrap_err().source().is_none());
    }

    #[tokio::test]
    async fn times_out_reporting_the_last_connection_error_as_cause_after_polling() {
        let budget_spanning_several_polls = ADMIN_POOL_POLL_INTERVAL * 4;

        let result = wait_until_postgres_admin_pool_ready(
            UNREACHABLE_ADMIN_URL,
            budget_spanning_several_polls,
        )
        .await;

        assert!(result.unwrap_err().source().is_some());
    }
}
