use std::sync::Arc;

use sqlx::PgPool;

use crate::cluster::Cluster;

pub struct Database {
    #[expect(
        dead_code,
        reason = "cluster Arc keeps the postgres container alive while this database is in use"
    )]
    cluster: Arc<Cluster>,
    database_url: String,
    db_name: String,
    pool: PgPool,
}

impl Database {
    #[doc(hidden)]
    #[must_use]
    pub fn new(cluster: Arc<Cluster>, database_url: String, db_name: String, pool: PgPool) -> Self {
        Self {
            cluster,
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
