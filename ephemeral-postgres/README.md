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
ephemeral-postgres = "0.4"
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

## Cleanup

Cleanup is automatic and tied to ownership — you never close a cluster manually:

- The container is stopped and removed as soon as the last `Arc<Cluster>` is dropped. Each
  `Database` holds an `Arc<Cluster>`, so the container outlives the databases carved from it and
  disappears once the cluster and all of its databases go out of scope at the end of the test.
- Keep the cluster in a local binding for the duration of the test. Never store a `Cluster` or
  `Database` in a `static`, `OnceLock`, or `lazy_static`: statics are never dropped, so the
  container would leak for the whole lifetime of the test process.
- If the test process is interrupted before those values drop — Ctrl-C, or a test runner killing
  it on timeout — a watchdog still stops and removes the containers on `SIGINT`, `SIGTERM`, and
  `SIGQUIT`.

## Sharing one server across test processes

Test runners such as `cargo nextest` run every test in its own process, so each test would start
its own container. When container startup dominates your test time, start one server for the
whole run with `ephemeral-postgres-run` and let every test carve its own database from it:

```sh
cargo install ephemeral-postgres-run --version 0.4.0 --locked
ephemeral-postgres-run postgres 18 -- cargo nextest run
```

`ephemeral-postgres-run <image-name> <image-tag> -- <command> [arguments...]` starts one
PostgreSQL container, waits until it accepts connections, runs the command with the server's
base URL in `EPHEMERAL_POSTGRES_URL`, removes the container, and exits with the command's exit
code. Tests attach with `Cluster::connect` and stay isolated through their own databases:

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

A connected cluster never stops the server; `ephemeral-postgres-run` owns its lifetime.

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
