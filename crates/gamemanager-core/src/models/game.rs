use serde::{Deserialize, Serialize};

/// The program used to start a game.
///
/// `Auto` is retained only to read v0.9 profile settings. New settings are
/// written with the resolved concrete runner by the profile service.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Runner {
    #[default]
    Auto,
    Native,
    Bottles,
    Nwjs,
    Mkxpz,
    External,
}

impl Runner {
    pub const fn is_legacy_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Native => "native",
            Self::Bottles => "bottles",
            Self::Nwjs => "nwjs",
            Self::Mkxpz => "mkxpz",
            Self::External => "external",
        }
    }
}

/// Per-game profile settings stored in `profiles/<profile_key>/settings.toml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub engine_type: String,
    pub entry_path: String,
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub runner: Runner,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_true")]
    pub sandbox_home: bool,
    #[serde(default)]
    pub bottle_name: Option<String>,
    #[serde(default)]
    pub cover_file: Option<String>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            engine_type: String::new(),
            entry_path: String::new(),
            runtime_version: None,
            runner: Runner::Auto,
            args: Vec::new(),
            sandbox_home: true,
            bottle_name: None,
            cover_file: None,
        }
    }
}

fn default_true() -> bool {
    true
}

/// A game row persisted in the v0.9 SQLite database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameRecord {
    pub id: String,
    pub profile_key: String,
    pub title: String,
    pub engine_type: String,
    pub game_path: String,
    pub normalized_path: String,
    pub game_type: String,
    pub detection_confidence: i32,
    pub runtime_version: Option<String>,
    pub cover_path: Option<String>,
    pub play_count: i64,
    pub metadata_json: Option<String>,
    pub created_at: i64,
    pub last_played_at: Option<i64>,
    pub updated_at: i64,
}

/// The game fields rendered in library views.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameSummary {
    pub id: String,
    pub title: String,
    pub engine_type: String,
    pub game_path: String,
    pub game_type: String,
    pub detection_confidence: i32,
    pub runtime_version: Option<String>,
    pub cover_path: Option<String>,
    pub play_count: i64,
    pub last_played_at: Option<i64>,
}

impl From<&GameRecord> for GameSummary {
    fn from(game: &GameRecord) -> Self {
        Self {
            id: game.id.clone(),
            title: game.title.clone(),
            engine_type: game.engine_type.clone(),
            game_path: game.game_path.clone(),
            game_type: game.game_type.clone(),
            detection_confidence: game.detection_confidence,
            runtime_version: game.runtime_version.clone(),
            cover_path: game.cover_path.clone(),
            play_count: game.play_count,
            last_played_at: game.last_played_at,
        }
    }
}
