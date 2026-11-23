use tokio::runtime::Builder;

use crate::ephemeral_postgres_tests_error::EphemeralPostgresTestsError;
use crate::test_result::TestResult;

/// # Errors
///
/// Fails when the runtime that runs the test cannot start, or when the test fails.
pub fn block_on<TFuture: Future<Output = TestResult>>(future: TFuture) -> TestResult {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|source| EphemeralPostgresTestsError::StartTestRuntime { source })?
        .block_on(future)
}
