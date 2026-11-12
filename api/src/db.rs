use sqlx::{postgres::PgPoolOptions, PgPool};

const SCHEMA: &str = include_str!("../migrations/001_init.sql");

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    for statement in SCHEMA.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        sqlx::query(statement).execute(&pool).await?;
    }
    Ok(pool)
}
