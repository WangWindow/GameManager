use serde::{Deserialize, Serialize};

/// 引擎数据传输对象（前端用，对应 db::schema::Engine）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine_type: String,
    pub path: String,
    pub installed_at: i64,
}
