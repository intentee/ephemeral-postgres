use sqlx::PgPool;
use sqlx::query_scalar;

pub async fn database_exists(pool: &PgPool, database_name: &str) -> bool {
    query_scalar("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
        .bind(database_name)
        .fetch_one(pool)
        .await
        .unwrap()
}
