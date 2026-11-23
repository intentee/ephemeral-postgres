use sqlx::query_scalar;

use ephemeral_postgres::ephemeral_database::EphemeralDatabase;
use ephemeral_postgres_tests::block_on::block_on;
use ephemeral_postgres_tests::test_result::TestResult;
use ephemeral_postgres_tests::verify::verify;

#[test]
fn provides_a_pool_connected_to_its_own_database() -> TestResult {
    block_on(async {
        let database = EphemeralDatabase::create().await?;
        let current_database: String = query_scalar("SELECT current_database()")
            .fetch_one(database.pool())
            .await?;

        verify(database.connect_options().get_database() == Some(current_database.as_str()))?;

        Ok(())
    })
}
