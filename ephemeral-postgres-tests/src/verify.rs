use std::panic::Location;

use crate::ephemeral_postgres_tests_error::EphemeralPostgresTestsError;

/// # Errors
///
/// Fails when the expectation does not hold, naming where the expectation is written.
#[track_caller]
pub const fn verify(holds: bool) -> Result<(), EphemeralPostgresTestsError> {
    if holds {
        Ok(())
    } else {
        Err(EphemeralPostgresTestsError::UnmetExpectation {
            location: Location::caller(),
        })
    }
}
