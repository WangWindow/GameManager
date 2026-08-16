/// An installed runtime or engine row persisted in the v0.9 SQLite database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine_type: String,
    pub engine_path: String,
    pub installed_at: i64,
}
