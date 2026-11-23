#[derive(Debug, thiserror::Error)]
pub enum EphemeralPostgresTestsError {
    #[error("failed to acquire advisory lock ({class_id}, {object_id})")]
    AcquireAdvisoryLock {
        class_id: i32,
        object_id: i32,
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to check whether database {database_name} exists")]
    CheckDatabaseExists {
        database_name: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to find the backend process of an advisory lock holder")]
    FindBackendPid {
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to find the advisory lock that backend {backend_pid} holds")]
    FindHeldAdvisoryLock {
        backend_pid: i32,
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to release advisory lock ({class_id}, {object_id})")]
    ReleaseAdvisoryLock {
        class_id: i32,
        object_id: i32,
        #[source]
        source: sqlx::Error,
    },

    #[error("failed to start the runtime that runs a test")]
    StartTestRuntime {
        #[source]
        source: std::io::Error,
    },

    #[error("the expectation at {location} does not hold")]
    UnmetExpectation {
        location: &'static std::panic::Location<'static>,
    },
}
