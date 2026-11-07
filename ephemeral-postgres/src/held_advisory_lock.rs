use sqlx::FromRow;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::query;
use sqlx::query_as;

#[derive(FromRow)]
pub struct HeldAdvisoryLock {
    pub class_id: i32,
    pub object_id: i32,
}

impl HeldAdvisoryLock {
    pub async fn held_by(pool: &PgPool, backend_pid: i32) -> Self {
        query_as(
            "SELECT classid::int4 AS class_id, objid::int4 AS object_id FROM pg_locks \
             WHERE locktype = 'advisory' AND objsubid = 2 AND pid = $1",
        )
        .bind(backend_pid)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    pub async fn wait_until_released(&self, connection: &mut PgConnection) {
        query("SELECT pg_advisory_lock($1, $2)")
            .bind(self.class_id)
            .bind(self.object_id)
            .execute(&mut *connection)
            .await
            .unwrap();
        query("SELECT pg_advisory_unlock($1, $2)")
            .bind(self.class_id)
            .bind(self.object_id)
            .execute(connection)
            .await
            .unwrap();
    }
}
