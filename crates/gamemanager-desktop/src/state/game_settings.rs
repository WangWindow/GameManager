use std::path::PathBuf;

use gamemanager_core::{GameConfig, GameSummary, Runner, RuntimeStatus, UpdateGameRequest};

use crate::ui::UiTokens;

#[derive(Clone, Debug)]
pub struct GameSettingsState {
    pub game_id: String,
    pub profile_key: String,
    pub title: String,
    pub game_path: String,
    pub entry_path: String,
    pub engine_type: String,
    pub runtime_version: Option<String>,
    pub runner: Runner,
    pub args: Vec<String>,
    pub sandbox_home: bool,
    pub bottle_name: Option<String>,
    pub cover_file: Option<String>,
    pub refreshing_cover: bool,
    pub error: Option<String>,
    pub saving: bool,
    cover_changed: bool,
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
            profile_key: String::new(),
            title: String::new(),
            game_path: String::new(),
            entry_path: String::new(),
            engine_type: String::new(),
            runtime_version: None,
            runner,
            args: Vec::new(),
            sandbox_home: true,
            bottle_name: None,
            cover_file: None,
            refreshing_cover: false,
            error: None,
            saving: false,
            cover_changed: false,
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
            profile_key: game.profile_key.clone(),
            title: game.title.clone(),
            game_path: game.game_path.clone(),
            entry_path: config.entry_path.clone(),
            engine_type: config.engine_type.clone(),
            runtime_version: config.runtime_version.clone(),
            runner,
            args: config.args.clone(),
            sandbox_home: config.sandbox_home,
            bottle_name: config.bottle_name.clone(),
            cover_file: config.cover_file.clone(),
            refreshing_cover: false,
            error: None,
            saving: false,
            cover_changed: false,
        }
    }

    pub fn shows_sandbox_home(&self) -> bool {
        matches!(self.runner, Runner::Native | Runner::Nwjs | Runner::Mkxpz)
    }

    pub fn natural_body_height(&self) -> f32 {
        let rows = self.visible_form_row_count() as f32;
        let items = rows + 1.0;
        rows * UiTokens::FORM_ROW_HEIGHT
            + (items - 1.0) * UiTokens::FORM_ROW_GAP
            + UiTokens::FORM_SECTION_HEIGHT
    }

    pub fn nwjs_versions(runtimes: &[RuntimeStatus]) -> Vec<String> {
        let mut versions = runtimes
            .iter()
            .filter(|runtime| runtime.engine_type.eq_ignore_ascii_case("nwjs"))
            .map(|runtime| runtime.version.clone())
            .filter(|version| !version.trim().is_empty())
            .collect::<Vec<_>>();
        versions.sort_by(|left, right| {
            numeric_version_components(right)
                .cmp(&numeric_version_components(left))
                .then_with(|| right.cmp(left))
        });
        versions.dedup();
        versions
    }

    pub fn select_bottle(&mut self, bottle: Option<String>) {
        self.bottle_name = bottle.filter(|name| !name.trim().is_empty());
    }

    pub fn set_engine_type(&mut self, engine_type: String) {
        self.engine_type = engine_type;
        if self.runner == Runner::Mkxpz
            && !matches!(self.engine_type.as_str(), "rpgmakervx" | "rpgmakervxace")
        {
            self.runner = Runner::Native;
        }
    }

    pub fn set_arguments_text(&mut self, args: String) {
        self.args = args
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
    }

    pub fn arguments_text(&self) -> String {
        self.args.join(" ")
    }

    pub fn set_cover_file(&mut self, cover_file: String) {
        self.cover_file = (!cover_file.trim().is_empty()).then_some(cover_file);
        self.cover_changed = true;
    }

    pub fn changed_cover_source(&self) -> Option<PathBuf> {
        self.cover_changed
            .then_some(self.cover_file.as_deref())
            .flatten()
            .map(PathBuf::from)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("游戏名称不能为空".to_owned());
        }
        if self.entry_path.trim().is_empty() {
            return Err("请选择入口文件或目录".to_owned());
        }
        Ok(())
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

    fn visible_form_row_count(&self) -> usize {
        let mut rows = 8;
        if matches!(self.runner, Runner::Nwjs | Runner::Bottles) {
            rows += 1;
        }
        if self.shows_sandbox_home() {
            rows += 1;
        }
        rows
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

fn numeric_version_components(version: &str) -> Vec<u32> {
    version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|component| !component.is_empty())
        .filter_map(|component| component.parse::<u32>().ok())
        .collect()
}

fn legacy_runner_for_engine(engine_type: &str) -> Runner {
    match engine_type {
        "html" | "rpgmakermv" | "rpgmakermz" => Runner::Nwjs,
        "rpgmakervx" | "rpgmakervxace" => Runner::Bottles,
        _ => Runner::Native,
    }
}
