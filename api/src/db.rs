use std::str::FromStr;
use std::time::Duration;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

const SCHEMA: &str = include_str!("../migrations/001_init.sql");

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    // 两条防卡死的保险：
    // 1) statement_cache_capacity(0)：若 URL 指向 Supabase 6543 事务池化，服务端不保留
    //    会话状态，带缓存的 prepared statement 会间歇报 "does not exist"；关掉对会话模式也无害。
    // 2) 首连限时 + acquire_timeout：直连端点 IPv6-only，路由一抖建连会挂死，连接池就报
    //    "pool timed out while waiting for an open connection"。已改走 IPv4 的 Pooler
    //    端点（见 api/.env 注释），这里再兜一层建连超时，挂死也会 20 秒内快速失败。
    let options = PgConnectOptions::from_str(url)?.statement_cache_capacity(0);
    let pool = tokio::time::timeout(
        Duration::from_secs(20),
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(60))
            .connect_with(options),
    )
    .await
    .map_err(|_| sqlx::Error::PoolTimedOut)??;
    for statement in split_statements(SCHEMA) {
        if skip_statement(statement) {
            continue;
        }
        sqlx::query(statement).execute(&pool).await?;
    }
    Ok(pool)
}

/// 把迁移 SQL 切成语句。`;` 在行注释、块注释、字符串和 `$$(函数体)$$` 里
/// 不是切分点，这样函数体和注释里的分号不会把语句切碎。空段直接丢掉。
fn split_statements(sql: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let bytes = sql.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'-' if bytes.get(i + 1) == Some(&b'-') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            b'\'' => {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\'' {
                        if bytes.get(i + 1) == Some(&b'\'') {
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            b'$' => {
                // $tag$ ... $tag$，tag 是空或标识符字符。
                let mut end = i + 1;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                if end < bytes.len() && bytes[end] == b'$' {
                    let tag = &sql[i..=end];
                    let body = end + 1;
                    i = match sql[body..].find(tag) {
                        Some(offset) => body + offset + tag.len(),
                        None => bytes.len(),
                    };
                } else {
                    i += 1;
                }
            }
            b';' => {
                let statement = sql[start..i].trim();
                if !statement.is_empty() {
                    statements.push(statement);
                }
                start = i + 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    let statement = sql[start..].trim();
    if !statement.is_empty() {
        statements.push(statement);
    }
    statements
}

/// 建函数、存储过程、触发器的语句不执行：迁移只需要表结构，
/// 而且运行账号通常没有建函数的权限，执行失败会拖垮整次建表。
fn skip_statement(statement: &str) -> bool {
    const SKIPPED: [&str; 6] = [
        "CREATE FUNCTION",
        "CREATE OR REPLACE FUNCTION",
        "CREATE PROCEDURE",
        "CREATE OR REPLACE PROCEDURE",
        "CREATE TRIGGER",
        "CREATE OR REPLACE TRIGGER",
    ];
    let head = statement.trim_start().to_ascii_uppercase();
    let skipped = SKIPPED.iter().any(|prefix| head.starts_with(prefix));
    if skipped {
        tracing::warn!("db: skipping function/trigger statement in migration");
    }
    skipped
}
