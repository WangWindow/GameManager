use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 引擎/运行时数据库记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Engine {
    /// 引擎唯一标识
    pub id: String,
    /// 引擎名称
    pub name: String,
    /// 引擎版本
    pub version: String,
    /// 引擎类型
    pub engine_type: String,
    /// 引擎安装路径
    pub path: String,
    /// 安装时间（Unix毫秒时间戳）
    pub installed_at: i64,
}

impl Engine {
    /// 创建新引擎记录
    pub fn new(
        id: String,
        name: String,
        version: String,
        engine_type: String,
        path: String,
    ) -> Self {
        let now = crate::utils::now_unix_ms();
        Self {
            id,
            name,
            version,
            engine_type,
            path,
            installed_at: now,
        }
    }
}

/// 引擎版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineVersion {
    /// 版本号
    pub version: String,
    /// 下载URL
    pub download_url: String,
    /// 是否已安装
    pub installed: bool,
}
