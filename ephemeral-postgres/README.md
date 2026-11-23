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
ephemeral-postgres = "0.6"
```

## Usage

```rust
use std::error::Error;

use ephemeral_postgres::ephemeral_database::EphemeralDatabase;
use sqlx::query;

#[tokio::test]
async fn each_test_gets_its_own_database() -> Result<(), Box<dyn Error>> {
    let database = EphemeralDatabase::create().await?;

    query("SELECT 1").execute(database.pool()).await?;

    Ok(())
}
```

## License

Apache-2.0.
