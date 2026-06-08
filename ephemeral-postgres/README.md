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

```rust
use ephemeral_postgres::cluster::Cluster;
use sqlx::Row;

#[tokio::test]
async fn each_test_gets_an_isolated_database() {
    let cluster = Cluster::start().await.unwrap();
    let database = cluster.create_database().await.unwrap();

    let value: i32 = sqlx::query("SELECT 1::int AS value")
        .fetch_one(database.pool())
        .await
        .unwrap()
        .get("value");

    assert_eq!(value, 1);
}
```

- `Cluster::start()` starts one PostgreSQL container.
- `create_database()` / `create_database_with_id(uuid)` create freshly-isolated databases that
  share the container.
- `database.pool()` returns an `sqlx::PgPool` connected to that database.
- Dropping the last `Arc<Cluster>` stops and removes the container.

## Configuration

```rust
use std::time::Duration;

use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;

let cluster = Cluster::start_with_params(ClusterParams {
    image_tag: "17".to_owned(),
    readiness_timeout: Duration::from_secs(60),
})
.await?;
```

`ClusterParams::default()` pins a specific `postgres` image digest and waits up to 30 seconds for
the server to accept connections.

## License

Apache-2.0.
