use std::time::{SystemTime, UNIX_EPOCH};

/// 获取当前Unix时间戳（毫秒）
pub fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
