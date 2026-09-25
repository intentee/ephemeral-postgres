use std::sync::Arc;

use sqlx::PgPool;

use crate::cluster_server::ClusterServer;

pub struct Database {
    #[expect(
        dead_code,
        reason = "server Arc keeps an owned postgres container alive while this database is in use"
    )]
    server: Arc<ClusterServer>,
    database_url: String,
    db_name: String,
    pool: PgPool,
}

impl Database {
    #[doc(hidden)]
    #[must_use]
    pub fn new(
        server: Arc<ClusterServer>,
        database_url: String,
        db_name: String,
        pool: PgPool,
    ) -> Self {
        Self {
            server,
            database_url,
            db_name,
            pool,
        }
    }

    #[must_use]
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
