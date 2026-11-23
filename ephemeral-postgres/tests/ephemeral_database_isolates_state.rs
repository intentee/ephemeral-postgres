use sqlx::Row;
use sqlx::query;

use ephemeral_postgres::ephemeral_database::EphemeralDatabase;
use ephemeral_postgres_tests::block_on::block_on;
use ephemeral_postgres_tests::has_sqlstate::has_sqlstate;
use ephemeral_postgres_tests::test_result::TestResult;
use ephemeral_postgres_tests::verify::verify;

#[test]
fn isolates_state_from_another_ephemeral_database() -> TestResult {
    block_on(async {
        let first = EphemeralDatabase::create().await?;
        let second = EphemeralDatabase::create().await?;

        query("CREATE TABLE isolation_check (value INT)")
            .execute(first.pool())
            .await?;
        query("INSERT INTO isolation_check (value) VALUES (1)")
            .execute(first.pool())
            .await?;

        let stored_value: i32 = query("SELECT value FROM isolation_check")
            .fetch_one(first.pool())
            .await?
            .try_get("value")?;

        verify(stored_value == 1)?;
        verify(matches!(
            query("SELECT value FROM isolation_check")
                .fetch_one(second.pool())
                .await,
            Err(error) if has_sqlstate(&error, "42P01")
        ))?;

        Ok(())
    })
}
