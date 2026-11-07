# ephemeral-postgres

Ephemeral PostgreSQL databases for Rust integration tests.

## Requirements

A PostgreSQL 15 or newer server provided by your environment, and its URL in
`EPHEMERAL_POSTGRES_URL`:

```sh
export EPHEMERAL_POSTGRES_URL='postgres://postgres@127.0.0.1:5432/postgres?sslmode=disable'
```

## Install

```toml
[dev-dependencies]
ephemeral-postgres = "0.5"
```

## Usage

```rust
use ephemeral_postgres::ephemeral_database::EphemeralDatabase;

#[tokio::test]
async fn each_test_gets_its_own_database() {
    let database = EphemeralDatabase::create().await.unwrap();

    sqlx::query("SELECT 1").execute(database.pool()).await.unwrap();
}
```

## License

Apache-2.0.
