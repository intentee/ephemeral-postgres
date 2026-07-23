use std::sync::Arc;
use std::time::Duration;

use sqlx::AssertSqlSafe;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::query;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use uuid::Uuid;

use crate::cluster_params::ClusterParams;
use crate::database::Database;
use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::postgres_container::PostgresContainer;
use crate::wait_until_postgres_admin_pool_ready::wait_until_postgres_admin_pool_ready;

const POSTGRES_HOST: &str = "127.0.0.1";

pub struct Cluster {
    admin_pool: PgPool,
    base_url: String,
    container: Arc<PostgresContainer>,
}

impl Cluster {
    pub async fn start(params: ClusterParams) -> Result<Self, EphemeralPostgresError> {
        let ClusterParams {
            image,
            readiness_timeout,
        } = params;

        let container = image
            .into_container_request()
            .start()
            .await
            .map_err(|source| EphemeralPostgresError::ContainerStart { source })?;

        Self::from_started_container(container, readiness_timeout).await
    }

    #[doc(hidden)]
    pub async fn from_started_container(
        container: ContainerAsync<Postgres>,
        readiness_timeout: Duration,
    ) -> Result<Self, EphemeralPostgresError> {
        let container = PostgresContainer::new(container);
        let port = container.mapped_host_port().await?;

        let base_url = format!("postgres://postgres@{POSTGRES_HOST}:{port}");
        let admin_pool = wait_until_postgres_admin_pool_ready(
            &format!("{base_url}/postgres"),
            readiness_timeout,
        )
        .await?;

        Ok(Self {
            admin_pool,
            base_url,
            container: Arc::new(container),
        })
    }

    pub async fn create_database(&self) -> Result<Database, EphemeralPostgresError> {
        self.create_database_with_id(Uuid::new_v4()).await
    }

    pub async fn create_database_with_id(
        &self,
        database_id: Uuid,
    ) -> Result<Database, EphemeralPostgresError> {
        let db_name = format!("test_{}", database_id.simple());

        query(AssertSqlSafe(format!("CREATE DATABASE \"{db_name}\"")))
            .execute(&self.admin_pool)
            .await
            .map_err(|source| EphemeralPostgresError::CreateDatabase {
                db_name: db_name.clone(),
                source,
            })?;

        self.open_database_pool(db_name).await
    }

    #[doc(hidden)]
    pub async fn open_database_pool(
        &self,
        db_name: String,
    ) -> Result<Database, EphemeralPostgresError> {
        let database_url = format!("{}/{db_name}", self.base_url);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .map_err(|source| EphemeralPostgresError::OpenPool {
                db_name: db_name.clone(),
                source,
            })?;

        Ok(Database::new(
            Arc::clone(&self.container),
            database_url,
            db_name,
            pool,
        ))
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
