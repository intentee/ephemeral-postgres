use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::query;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use uuid::Uuid;

use crate::cluster_params::ClusterParams;
use crate::database::Database;
use crate::wait_until_postgres_admin_pool_ready::wait_until_postgres_admin_pool_ready;

const POSTGRES_HOST: &str = "127.0.0.1";
const POSTGRES_CONTAINER_PORT: u16 = 5432;

async fn start_postgres_container(image_tag: String) -> Result<ContainerAsync<Postgres>> {
    Postgres::default()
        .with_host_auth()
        .with_tag(image_tag)
        .start()
        .await
        .context("failed to start ephemeral postgres container")
}

pub struct Cluster {
    admin_pool: PgPool,
    base_url: String,
    #[expect(
        dead_code,
        reason = "container handle keeps the postgres container alive until the cluster Arc is dropped"
    )]
    container: ContainerAsync<Postgres>,
}

impl Cluster {
    pub async fn start() -> Result<Arc<Self>> {
        Self::start_with_params(ClusterParams::default()).await
    }

    pub async fn start_with_params(params: ClusterParams) -> Result<Arc<Self>> {
        let ClusterParams {
            image_tag,
            readiness_timeout,
        } = params;

        let container = start_postgres_container(image_tag).await?;

        Self::from_started_container(container, readiness_timeout).await
    }

    async fn from_started_container(
        container: ContainerAsync<Postgres>,
        readiness_timeout: Duration,
    ) -> Result<Arc<Self>> {
        let port = container
            .get_host_port_ipv4(POSTGRES_CONTAINER_PORT)
            .await
            .context("failed to resolve ephemeral postgres mapped port")?;

        let base_url = format!("postgres://postgres@{POSTGRES_HOST}:{port}");
        let admin_pool = wait_until_postgres_admin_pool_ready(
            &format!("{base_url}/postgres"),
            readiness_timeout,
        )
        .await
        .context("postgres admin pool did not become ready before timeout")?;

        Ok(Arc::new(Self {
            admin_pool,
            base_url,
            container,
        }))
    }

    pub async fn create_database(self: &Arc<Self>) -> Result<Database> {
        self.create_database_with_id(Uuid::new_v4()).await
    }

    pub async fn create_database_with_id(self: &Arc<Self>, database_id: Uuid) -> Result<Database> {
        let db_name = format!("test_{}", database_id.simple());

        query(&format!("CREATE DATABASE \"{db_name}\""))
            .execute(&self.admin_pool)
            .await
            .context("failed to create per-test ephemeral database")?;

        let database_url = format!("{}/{db_name}", self.base_url);

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .context("failed to open pool against per-test ephemeral database")
            .map(|pool| Database::new(Arc::clone(self), database_url, db_name, pool))
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::Cluster;
    use super::start_postgres_container;
    use crate::cluster_params::ClusterParams;

    #[tokio::test]
    async fn from_started_container_errors_when_mapped_port_unavailable() {
        let ClusterParams {
            image_tag,
            readiness_timeout,
        } = ClusterParams::default();

        let container = start_postgres_container(image_tag)
            .await
            .expect("postgres container should start");
        container
            .stop_with_timeout(Some(0))
            .await
            .expect("postgres container should stop before finalizing the cluster");

        let result = Cluster::from_started_container(container, readiness_timeout).await;

        assert!(
            result.is_err(),
            "finalizing a cluster must fail when the container's mapped port is unavailable",
        );
    }
}
