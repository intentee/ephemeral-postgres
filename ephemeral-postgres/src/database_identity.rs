use sqlx::AssertSqlSafe;
use sqlx::Connection;
use sqlx::PgConnection;
use sqlx::postgres::PgConnectOptions;
use sqlx::query;
use sqlx::raw_sql;
use tokio::runtime::Builder;

use crate::ephemeral_postgres_error::EphemeralPostgresError;
use crate::lease_stage::LeaseStage;

const NAME_PREFIX: &str = "ephemeral_postgres_";

pub struct DatabaseIdentity {
    pub oid: i32,
}

impl DatabaseIdentity {
    pub async fn create(
        &self,
        connection: &mut PgConnection,
    ) -> Result<(), EphemeralPostgresError> {
        raw_sql(AssertSqlSafe(format!(
            "CREATE DATABASE \"{}\" OID {} TEMPLATE template0",
            self.name(),
            self.oid
        )))
        .execute(connection)
        .await
        .map_err(|source| EphemeralPostgresError::CreateDatabase {
            database_name: self.name(),
            source,
        })?;

        Ok(())
    }

    pub async fn drop_database(&self, connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        raw_sql(AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.name()
        )))
        .execute(connection)
        .await?;

        Ok(())
    }

    #[must_use]
    pub fn name(&self) -> String {
        format!("{NAME_PREFIX}{}", self.oid)
    }

    pub fn release_in_own_runtime(&self, admin_options: &PgConnectOptions, stage: &LeaseStage) {
        if let Err(error) = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(EphemeralPostgresError::CleanupRuntime)
            .and_then(|runtime| runtime.block_on(self.release(admin_options, stage)))
        {
            eprintln!("ephemeral-postgres: {error:?}");
        }
    }

    async fn release(
        &self,
        admin_options: &PgConnectOptions,
        stage: &LeaseStage,
    ) -> Result<(), EphemeralPostgresError> {
        let mut connection = PgConnection::connect_with(admin_options)
            .await
            .map_err(|source| EphemeralPostgresError::AdminConnection { source })?;

        if matches!(stage, LeaseStage::Created) {
            self.drop_database(&mut connection)
                .await
                .map_err(|source| EphemeralPostgresError::DropDatabase {
                    database_name: self.name(),
                    source,
                })?;
        }

        query("DELETE FROM ephemeral_postgres.databases WHERE database_oid = $1")
            .bind(self.oid)
            .execute(&mut connection)
            .await
            .map_err(|source| EphemeralPostgresError::UnregisterDatabase {
                database_name: self.name(),
                source,
            })?;

        Ok(())
    }
}
