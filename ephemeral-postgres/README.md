# ephemeral-postgres

Ephemeral PostgreSQL databases for Rust integration tests.

## Requirements

A running Docker daemon.

## Install

```toml
[dev-dependencies]
ephemeral-postgres = "0.5"
```

## Usage

```rust
use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::postgres_image::PostgresImage;
use sqlx::Row;

#[tokio::test]
async fn each_test_gets_an_isolated_database() {
    let cluster = Cluster::start(ClusterParams::new(PostgresImage::new("postgres", "18")))
        .await
        .unwrap();
    let database = cluster.create_database().await.unwrap();

    let value: i32 = sqlx::query("SELECT 1::int AS value")
        .fetch_one(database.pool())
        .await
        .unwrap()
        .get("value");

    assert_eq!(value, 1);
}
```

## License

Apache-2.0.
