# ephemeral-postgres

Ephemeral PostgreSQL databases for Rust integration tests.

## Requirements

A running Docker daemon.

## Install

```toml
[dev-dependencies]
ephemeral-postgres = "0.4"
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

## Sharing one server across test processes

```sh
cargo install ephemeral-postgres-run --version 0.4.0 --locked
ephemeral-postgres-run postgres 18 -- cargo nextest run
```

```rust
use std::env;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_connection_params::ClusterConnectionParams;

#[tokio::test]
async fn each_test_gets_an_isolated_database_on_the_shared_server() {
    let cluster = Cluster::connect(ClusterConnectionParams::new(
        env::var("EPHEMERAL_POSTGRES_URL").unwrap(),
    ))
    .await
    .unwrap();
    let database = cluster.create_database().await.unwrap();

    sqlx::query("SELECT 1").execute(database.pool()).await.unwrap();
}
```

## License

Apache-2.0.
