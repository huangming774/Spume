use std::str::FromStr;
use std::time::Duration;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

const SCHEMA: &str = include_str!("../migrations/001_init.sql");

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    // Supabase 6543 是事务池化（transaction pooling）模式，服务端不保留会话状态，
    // 必须关闭 sqlx 的 prepared statement 缓存，否则会间歇报 "prepared statement ... does not exist"。
    let options = PgConnectOptions::from_str(url)?.statement_cache_capacity(0);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .connect_with(options)
        .await?;
    for statement in SCHEMA.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        sqlx::query(statement).execute(&pool).await?;
    }
    Ok(pool)
}
