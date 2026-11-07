use std::str::Utf8Error;

#[derive(Debug, thiserror::Error)]
pub enum EphemeralPostgresError {
    #[error("failed to connect to the admin database")]
    AdminConnection {
        #[source]
        source: sqlx::Error,
    },

    #[error("the server url holds more than one path segment: `{path}`")]
    AmbiguousDatabasePath { path: String },

    #[error("failed to start the runtime that removes an ephemeral database")]
    CleanupRuntime(#[source] std::io::Error),

    #[error("{variables:?} would describe the server in addition to the server url; unset them")]
    ConflictingEnvironment { variables: Vec<&'static str> },

    #[error("failed to create ephemeral database {database_name}")]
    CreateDatabase {
        database_name: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to drop ephemeral database {database_name}")]
    DropDatabase {
        database_name: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("the database name in the server url is not valid UTF-8")]
    DatabaseNameNotUnicode {
        #[source]
        source: Utf8Error,
    },

    #[error("the server url names no database")]
    MissingDatabase,

    #[error("the server url names no host")]
    MissingHost,

    #[error("the server url names no port")]
    MissingPort,

    #[error("{variable} is not set")]
    MissingServerUrl { variable: &'static str },

    #[error("the server url sets no sslmode")]
    MissingSslMode,

    #[error("the server url names no user")]
    MissingUsername,

    #[error("failed to open a pool against ephemeral database {database_name}")]
    OpenPool {
        database_name: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("the password in the server url is not valid UTF-8")]
    PasswordNotUnicode {
        #[source]
        source: Utf8Error,
    },

    #[error("failed to prepare the ephemeral database registry")]
    PrepareRegistry {
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to reclaim ephemeral databases whose owners are gone")]
    ReclaimOrphans {
        #[source]
        source: sqlx::Error,
    },

    #[error("the server url sets sslmode more than once")]
    RepeatedSslMode,

    #[error("failed to reserve an ephemeral database")]
    ReserveDatabase {
        #[source]
        source: sqlx::Error,
    },

    #[error("{variable} is not valid unicode")]
    ServerUrlNotUnicode { variable: &'static str },

    #[error("failed to unregister ephemeral database {database_name}")]
    UnregisterDatabase {
        database_name: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("the server url has a fragment")]
    UnexpectedFragment,

    #[error("the server url has an unsupported query parameter `{name}`")]
    UnexpectedQueryParameter { name: String },

    #[error("the server url sslmode `{value}` is not one of the libpq modes")]
    UnknownSslMode { value: String },

    #[error("failed to parse the server url")]
    UnparsableServerUrl {
        #[source]
        source: url::ParseError,
    },

    #[error("the server url scheme is `{scheme}`, expected `postgres`")]
    UnsupportedScheme { scheme: String },

    #[error("the user in the server url is not valid UTF-8")]
    UsernameNotUnicode {
        #[source]
        source: Utf8Error,
    },
}
