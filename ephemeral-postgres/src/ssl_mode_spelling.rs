use sqlx::postgres::PgSslMode;

pub struct SslModeSpelling {
    pub mode: PgSslMode,
    pub spelling: &'static str,
}
