use std::sync::Arc;

use sqlx::PgPool;

use crate::postgres_container::PostgresContainer;

pub struct Database {
    _container: Arc<PostgresContainer>,
    url: String,
    db_name: String,
    pool: PgPool,
}

impl Database {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        container: Arc<PostgresContainer>,
        database_url: String,
        db_name: String,
        pool: PgPool,
    ) -> Self {
        Self {
            _container: container,
            url: database_url,
            db_name,
            pool,
        }
    }

    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.url
    }

    #[must_use]
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}
