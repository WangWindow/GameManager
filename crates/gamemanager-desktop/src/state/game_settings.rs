use std::path::PathBuf;

use gamemanager_core::{GameConfig, GameSummary, Runner, UpdateGameRequest};

#[derive(Clone, Debug)]
pub struct GameSettingsState {
    pub game_id: String,
    pub title: String,
    pub entry_path: String,
    pub engine_type: String,
    pub runtime_version: Option<String>,
    pub runner: Runner,
    pub args: Vec<String>,
    pub sandbox_home: bool,
    pub bottle_name: Option<String>,
    pub cover_file: Option<String>,
    pub error: Option<String>,
    pub saving: bool,
}

#[derive(Clone, Debug)]
pub struct GameSettingsUpdate {
    pub game: UpdateGameRequest,
    pub config: GameConfig,
}

impl GameSettingsState {
    pub fn with_runner(runner: Runner) -> Self {
        Self {
            game_id: String::new(),
            title: String::new(),
            entry_path: String::new(),
            engine_type: String::new(),
            runtime_version: None,
            runner,
            args: Vec::new(),
            sandbox_home: true,
            bottle_name: None,
            cover_file: None,
            error: None,
            saving: false,
        }
    }

    pub fn from_game_and_config(game: &GameSummary, config: &GameConfig) -> Self {
        let runner = if config.runner == Runner::Auto {
            legacy_runner_for_engine(&game.engine_type)
        } else {
            config.runner
        };
        Self {
            game_id: game.id.clone(),
            title: game.title.clone(),
            entry_path: config.entry_path.clone(),
            engine_type: config.engine_type.clone(),
            runtime_version: config.runtime_version.clone(),
            runner,
            args: config.args.clone(),
            sandbox_home: config.sandbox_home,
            bottle_name: config.bottle_name.clone(),
            cover_file: config.cover_file.clone(),
            error: None,
            saving: false,
        }
    }

    pub fn shows_sandbox_home(&self) -> bool {
        matches!(self.runner, Runner::Native | Runner::Nwjs | Runner::Mkxpz)
    }

    pub fn runner_choices(
        &self,
        bottles_enabled: bool,
        nwjs_available: bool,
        mkxpz_available: bool,
    ) -> Vec<Runner> {
        let mut choices = vec![Runner::Native];
        if bottles_enabled {
            choices.push(Runner::Bottles);
        }
        if nwjs_available {
            choices.push(Runner::Nwjs);
        }
        if mkxpz_available && matches!(self.engine_type.as_str(), "rpgmakervx" | "rpgmakervxace") {
            choices.push(Runner::Mkxpz);
        }
        choices.push(Runner::External);
        choices
    }

    pub fn into_update_request(&self) -> GameSettingsUpdate {
        let config = GameConfig {
            engine_type: self.engine_type.clone(),
            entry_path: self.entry_path.clone(),
            runtime_version: self.runtime_version.clone(),
            runner: self.runner,
            args: self.args.clone(),
            sandbox_home: self.sandbox_home,
            bottle_name: self.bottle_name.clone(),
            cover_file: self.cover_file.clone(),
        };
        GameSettingsUpdate {
            game: UpdateGameRequest {
                title: Some(self.title.clone()),
                entry: (!self.entry_path.trim().is_empty())
                    .then(|| PathBuf::from(&self.entry_path)),
                engine_id: Some(self.engine_type.clone()),
                runtime_version: self.runtime_version.clone(),
                metadata_json: None,
            },
            config,
        }
    }
}

fn legacy_runner_for_engine(engine_type: &str) -> Runner {
    match engine_type {
        "html" | "rpgmakermv" | "rpgmakermz" => Runner::Nwjs,
        "rpgmakervx" | "rpgmakervxace" => Runner::Bottles,
        _ => Runner::Native,
    }
}
