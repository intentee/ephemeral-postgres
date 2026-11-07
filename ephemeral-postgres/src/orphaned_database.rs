use sqlx::FromRow;

#[derive(FromRow)]
pub struct OrphanedDatabase {
    pub database_name: Option<String>,
    pub database_oid: i32,
}
