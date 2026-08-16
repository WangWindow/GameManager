use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Child, Command},
};

use crate::{BottlesCli, CoreError, EngineRegistry, GameConfig, GameRecord, Result, Runner};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchPlan {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub current_dir: PathBuf,
    pub env: BTreeMap<OsString, OsString>,
    runner: Runner,
}

impl LaunchPlan {
    pub fn runner(&self) -> Runner {
        self.runner
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args).current_dir(&self.current_dir);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        command
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchResult {
    pub pid: u32,
}

#[derive(Clone, Debug)]
pub struct Launcher {
    container_root: PathBuf,
    registry: EngineRegistry,
    bottles_cli: Option<BottlesCli>,
    nwjs_runtime: Option<PathBuf>,
    mkxpz_executable: Option<PathBuf>,
}

impl Launcher {
    pub fn new(container_root: impl Into<PathBuf>) -> Self {
        Self {
            container_root: container_root.into(),
            registry: EngineRegistry::default(),
            bottles_cli: None,
            nwjs_runtime: None,
            mkxpz_executable: None,
        }
    }

    pub fn with_registry(mut self, registry: EngineRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn with_bottles_cli(mut self, cli: BottlesCli) -> Self {
        self.bottles_cli = Some(cli);
        self
    }

    pub fn with_nwjs_runtime(mut self, runtime: impl Into<PathBuf>) -> Self {
        self.nwjs_runtime = Some(runtime.into());
        self
    }

    pub fn with_mkxpz_executable(mut self, executable: impl Into<PathBuf>) -> Self {
        self.mkxpz_executable = Some(executable.into());
        self
    }

    pub fn plan(&self, game: &GameRecord, config: &GameConfig) -> Result<LaunchPlan> {
        let game_root = PathBuf::from(&game.game_path);
        if !game_root.is_dir() {
            return Err(CoreError::InvalidPath(format!(
                "game directory does not exist: {}",
                game_root.display()
            )));
        }
        let runner = self.resolve_runner(game, config);
        match runner {
            Runner::Native => self.plan_native(game, config, &game_root),
            Runner::Bottles => self.plan_bottles(game, config, &game_root),
            Runner::Nwjs => self.plan_nwjs(game, config, &game_root),
            Runner::Mkxpz => self.plan_mkxpz(game, config, &game_root),
            Runner::External => self.plan_external(game, config, &game_root),
            Runner::Auto => Err(CoreError::Configuration(
                "runner must be resolved before launch".to_owned(),
            )),
        }
    }

    pub fn spawn(&self, plan: LaunchPlan) -> Result<LaunchResult> {
        let mut command = plan.command();
        let child = command.spawn().map_err(|error| {
            CoreError::Engine(format!(
                "failed to start {}: {error}",
                plan.program.display()
            ))
        })?;
        Ok(LaunchResult { pid: child.id() })
    }

    fn resolve_runner(&self, game: &GameRecord, config: &GameConfig) -> Runner {
        if !config.runner.is_legacy_auto() {
            return config.runner;
        }
        match self
            .registry
            .profile(&game.engine_type)
            .map(|profile| profile.launch.strategy.as_str())
        {
            Some("native") => Runner::Native,
            Some("nwjs") => Runner::Nwjs,
            Some("mkxpz") => Runner::Mkxpz,
            Some("external") => Runner::External,
            _ => Runner::Bottles,
        }
    }

    fn plan_native(
        &self,
        game: &GameRecord,
        config: &GameConfig,
        root: &Path,
    ) -> Result<LaunchPlan> {
        let entry = self
            .resolve_entry(root, &config.entry_path)?
            .ok_or_else(|| {
                CoreError::InvalidPath("native runner requires an entry file".to_owned())
            })?;
        let mut plan = self.base_plan(Runner::Native, &entry, root, config.args.clone());
        self.apply_home_sandbox(&mut plan, game, config);
        Ok(plan)
    }

    fn plan_bottles(
        &self,
        game: &GameRecord,
        config: &GameConfig,
        root: &Path,
    ) -> Result<LaunchPlan> {
        let entry = self
            .resolve_entry(root, &config.entry_path)?
            .ok_or_else(|| {
                CoreError::InvalidPath("Bottles runner requires an entry file".to_owned())
            })?;
        let bottle = config
            .bottle_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .ok_or_else(|| {
                CoreError::Configuration("Bottles runner requires a bottle".to_owned())
            })?;
        let cli = self
            .bottles_cli
            .as_ref()
            .ok_or_else(|| CoreError::Configuration("Bottles CLI is not configured".to_owned()))?;
        let (program, args) = cli.plan_run(bottle, &entry, &config.args);
        let mut plan = LaunchPlan {
            program,
            args,
            current_dir: root.to_path_buf(),
            env: BTreeMap::new(),
            runner: Runner::Bottles,
        };
        self.apply_home_sandbox(&mut plan, game, config);
        Ok(plan)
    }

    fn plan_nwjs(&self, game: &GameRecord, config: &GameConfig, root: &Path) -> Result<LaunchPlan> {
        let runtime = self.nwjs_runtime.as_ref().ok_or_else(|| {
            CoreError::Configuration("NW.js runtime is not configured".to_owned())
        })?;
        let program = find_runtime_binary(runtime, &["nw", "nwjs", "nw.exe", "nwjs.exe"])?;
        let entry = self.resolve_entry(root, &config.entry_path)?;
        let app = entry.as_deref().map_or_else(
            || root.to_path_buf(),
            |path| {
                if path.is_file()
                    && path.extension().is_some_and(|extension| {
                        extension.eq_ignore_ascii_case("html")
                            || extension.eq_ignore_ascii_case("htm")
                    })
                {
                    path.parent().unwrap_or(root).to_path_buf()
                } else {
                    path.to_path_buf()
                }
            },
        );
        let mut args = config.args.iter().map(OsString::from).collect::<Vec<_>>();
        if config.sandbox_home {
            let user_data = self.user_data_dir(&game.profile_key)?;
            let crash = self.crash_dir(&game.profile_key)?;
            args.push(OsString::from(format!(
                "--user-data-dir={}",
                user_data.display()
            )));
            args.push(OsString::from(format!(
                "--crash-dumps-dir={}",
                crash.display()
            )));
            let mut plan = LaunchPlan {
                program,
                args,
                current_dir: root.to_path_buf(),
                env: BTreeMap::new(),
                runner: Runner::Nwjs,
            };
            plan.env.insert(
                OsString::from("BREAKPAD_DUMP_LOCATION"),
                crash.into_os_string(),
            );
            plan.args.push(app.into_os_string());
            return Ok(plan);
        }
        args.push(app.into_os_string());
        Ok(LaunchPlan {
            program,
            args,
            current_dir: root.to_path_buf(),
            env: BTreeMap::new(),
            runner: Runner::Nwjs,
        })
    }

    fn plan_mkxpz(
        &self,
        game: &GameRecord,
        config: &GameConfig,
        root: &Path,
    ) -> Result<LaunchPlan> {
        if !matches!(game.engine_type.as_str(), "rpgmakervx" | "rpgmakervxace") {
            return Err(CoreError::Configuration(
                "mkxp-z requires RPG Maker VX / VX Ace".to_owned(),
            ));
        }
        let executable = self.mkxpz_executable.as_ref().ok_or_else(|| {
            CoreError::Configuration("mkxp-z runtime is not configured".to_owned())
        })?;
        if !executable.is_file() {
            return Err(CoreError::InvalidPath(format!(
                "mkxp-z executable does not exist: {}",
                executable.display()
            )));
        }
        let profile_dir = self.profile_dir(&game.profile_key).join("mkxpz");
        std::fs::create_dir_all(&profile_dir)?;
        let runtime_dir = executable.parent().unwrap_or(executable);
        let patch = runtime_dir.join("patches/compatibility.rb");
        let mut json = serde_json::json!({ "gameFolder": root.to_string_lossy() });
        if patch.is_file() {
            json["preloadScript"] = serde_json::json!([patch.to_string_lossy()]);
        }
        std::fs::write(
            profile_dir.join("mkxp.json"),
            serde_json::to_vec_pretty(&json)
                .map_err(|error| CoreError::Configuration(error.to_string()))?,
        )?;
        let mut plan = LaunchPlan {
            program: executable.clone(),
            args: config.args.iter().map(OsString::from).collect(),
            current_dir: profile_dir.clone(),
            env: BTreeMap::new(),
            runner: Runner::Mkxpz,
        };
        plan.env
            .insert(OsString::from("SRCDIR"), profile_dir.into_os_string());
        self.apply_home_sandbox(&mut plan, game, config);
        Ok(plan)
    }

    fn plan_external(
        &self,
        game: &GameRecord,
        config: &GameConfig,
        root: &Path,
    ) -> Result<LaunchPlan> {
        let entry = self
            .resolve_entry(root, &config.entry_path)?
            .ok_or_else(|| {
                CoreError::InvalidPath("external runner requires an entry file".to_owned())
            })?;
        let launch = self
            .registry
            .profile(&game.engine_type)
            .map(|profile| &profile.launch)
            .ok_or_else(|| {
                CoreError::Configuration(format!(
                    "no external launch profile for {}",
                    game.engine_type
                ))
            })?;
        if launch.program.trim().is_empty() {
            return Err(CoreError::Configuration(
                "external launch program is empty".to_owned(),
            ));
        }
        let template = if launch.args_template.trim().is_empty() {
            "{exe}"
        } else {
            launch.args_template.as_str()
        };
        let resolved = template
            .replace("{exe}", &entry.to_string_lossy())
            .replace("{game_dir}", &root.to_string_lossy());
        let mut args = launch
            .program_args_prefix
            .iter()
            .map(OsString::from)
            .collect::<Vec<_>>();
        args.extend(
            resolved
                .split_whitespace()
                .filter(|part| !part.is_empty())
                .map(OsString::from),
        );
        args.extend(config.args.iter().map(OsString::from));
        let mut plan = LaunchPlan {
            program: PathBuf::from(&launch.program),
            args,
            current_dir: root.to_path_buf(),
            env: BTreeMap::new(),
            runner: Runner::External,
        };
        self.apply_home_sandbox(&mut plan, game, config);
        Ok(plan)
    }

    fn base_plan(
        &self,
        runner: Runner,
        entry: &Path,
        root: &Path,
        args: Vec<String>,
    ) -> LaunchPlan {
        LaunchPlan {
            program: entry.to_path_buf(),
            args: args.into_iter().map(OsString::from).collect(),
            current_dir: root.to_path_buf(),
            env: BTreeMap::new(),
            runner,
        }
    }

    fn resolve_entry(&self, root: &Path, entry: &str) -> Result<Option<PathBuf>> {
        if entry.trim().is_empty() {
            return Ok(None);
        }
        let path = PathBuf::from(entry);
        let candidate = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        if candidate.is_file() {
            Ok(Some(
                std::fs::canonicalize(candidate).unwrap_or_else(|_| PathBuf::from(entry)),
            ))
        } else {
            Err(CoreError::InvalidPath(format!(
                "entry does not exist: {}",
                candidate.display()
            )))
        }
    }

    fn apply_home_sandbox(&self, plan: &mut LaunchPlan, game: &GameRecord, config: &GameConfig) {
        if config.sandbox_home
            && let Ok(user_data) = self.user_data_dir(&game.profile_key)
        {
            plan.env
                .insert(OsString::from("HOME"), user_data.into_os_string());
        }
    }

    fn profile_dir(&self, profile_key: &str) -> PathBuf {
        self.container_root.join("profiles").join(profile_key)
    }
    fn user_data_dir(&self, profile_key: &str) -> Result<PathBuf> {
        let path = self.profile_dir(profile_key).join("User Data");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }
    fn crash_dir(&self, profile_key: &str) -> Result<PathBuf> {
        let path = self.profile_dir(profile_key).join("Crash Reports");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }
}

fn find_runtime_binary(runtime: &Path, names: &[&str]) -> Result<PathBuf> {
    names
        .iter()
        .map(|name| runtime.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            CoreError::InvalidPath(format!("runtime binary not found in {}", runtime.display()))
        })
}

#[allow(dead_code)]
fn _child_id(child: &Child) -> u32 {
    child.id()
}
