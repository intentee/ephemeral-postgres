# ephemeral-postgres

Ephemeral PostgreSQL instances for Rust integration tests, backed by
[testcontainers](https://crates.io/crates/testcontainers) (Docker).

Each test starts its own PostgreSQL container and carves out isolated databases from it, so
tests never share state. Containers are removed automatically when the cluster is dropped.

## Requirements

A running Docker daemon. Docker is needed only at **test runtime** to start containers — it is
not required to build the crate or its documentation.

## Install

```toml
[dev-dependencies]
ephemeral-postgres = "0.1"
```

## Usage

You choose the image — `ephemeral-postgres` never assumes one. Pass any
postgres-protocol-compatible image (the official `postgres`, `postgis/postgis`,
`timescale/timescaledb`, a private-registry mirror, …) as a name and tag.

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

- `Cluster::start(params)` starts one PostgreSQL container from the image you provide.
- `create_database()` / `create_database_with_id(uuid)` create freshly-isolated databases that
  share the container.
- `database.pool()` returns an `sqlx::PgPool` connected to that database.
- Dropping the last `Arc<Cluster>` stops and removes the container.

## Configuration

`ClusterParams::new(image)` waits up to 30 seconds for the server to accept connections. Override
the readiness timeout with struct-update syntax:

```rust
use std::time::Duration;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;
use ephemeral_postgres::postgres_image::PostgresImage;

let cluster = Cluster::start(ClusterParams {
    readiness_timeout: Duration::from_secs(60),
    ..ClusterParams::new(PostgresImage::new("postgres", "18"))
})
.await?;
```

## License

Apache-2.0.
