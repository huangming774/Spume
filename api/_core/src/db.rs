use std::str::FromStr;
use std::time::Duration;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

const SCHEMA: &str = include_str!("../migrations/001_init.sql");

pub async fn connect(
    url: &str,
    acquire: Duration,
    statement: Duration,
) -> Result<PgPool, sqlx::Error> {
    let statement_ms = statement.as_millis().max(1).to_string();
    let options = PgConnectOptions::from_str(url)?
        .statement_cache_capacity(0)
        .options([("statement_timeout", statement_ms.as_str())]);
    let pool = tokio::time::timeout(
        Duration::from_secs(20),
        PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(acquire)
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
