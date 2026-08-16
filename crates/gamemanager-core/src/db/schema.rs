use toasty::Model;

#[derive(Clone, Debug, Model)]
#[table = "games"]
pub(crate) struct Game {
    #[key]
    pub id: String,
    pub profile_key: String,
    pub title: String,
    pub engine_type: String,
    #[column("path")]
    pub game_path: String,
    #[unique]
    pub normalized_path: String,
    #[default("unknown".to_string())]
    pub game_type: String,
    #[default(0)]
    pub detection_confidence: i32,
    pub runtime_version: Option<String>,
    pub cover_path: Option<String>,
    #[default(0)]
    pub play_count: i64,
    pub metadata_json: Option<String>,
    pub created_at: i64,
    pub last_played_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Model)]
#[table = "engines"]
pub(crate) struct Engine {
    #[key]
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine_type: String,
    #[column("path")]
    pub engine_path: String,
    pub installed_at: i64,
}

#[derive(Clone, Debug, Model)]
#[table = "settings"]
pub(crate) struct Setting {
    #[key]
    pub key: String,
    pub value: String,
}
