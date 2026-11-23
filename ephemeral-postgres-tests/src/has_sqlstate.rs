use sqlx::Error;
use sqlx::error::DatabaseError;

#[must_use]
pub fn has_sqlstate(error: &Error, sqlstate: &str) -> bool {
    error
        .as_database_error()
        .and_then(DatabaseError::code)
        .is_some_and(|code| code == sqlstate)
}
