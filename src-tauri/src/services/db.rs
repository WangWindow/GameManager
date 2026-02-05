// 数据库模块
// 负责数据库连接和基础操作

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

/// 初始化数据库连接池
pub async fn init_database(db_path: &Path) -> Result<SqlitePool, String> {
    // 确保数据库目录存在
    if let Some(parent) = db_path.parent() {
        crate::services::path::ensure_dir(parent)?;
    }

    // 创建连接选项
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    // 创建连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| format!("数据库连接失败: {}", e))?;

    // 运行数据库迁移
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| format!("数据库迁移失败: {}", e))?;

    Ok(pool)
}

/// 获取设置值
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询设置失败: {}", e))?;
    Ok(row.map(|r| r.0))
}

/// 设置值
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings(key, value) VALUES(?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| format!("设置保存失败: {}", e))?;
    Ok(())
}
