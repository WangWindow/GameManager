use crate::models::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// 游戏启动服务
pub struct LauncherService {
    file_service: crate::services::FileService,
}

struct LaunchOptions {
    entry_path: Option<String>,
    args: Vec<String>,
    sandbox_home: bool,
}

impl LauncherService {
    /// 创建启动服务实例
    pub fn new() -> Self {
        Self {
            file_service: crate::services::FileService::new(),
        }
    }

    /// 启动游戏
    pub async fn launch_game(
        &self,
        game: &Game,
        container_root: &Path,
        nwjs_runtime_dir: Option<&Path>,
        config: Option<&GameConfig>,
    ) -> Result<LaunchResult, String> {
        // 检查游戏路径是否存在
        let game_path = Path::new(&game.path);
        if !game_path.exists() {
            return Err(format!("游戏路径不存在: {}", game.path));
        }

        // 确保游戏目录结构存在
        self.file_service
            .ensure_game_dirs(container_root, &game.profile_key)?;

        let options = self.resolve_launch_options(config);

        // 根据引擎类型启动游戏
        let engine_type = game.get_engine_type();
        let child = match engine_type {
            EngineType::RpgMakerVX | EngineType::RpgMakerVXAce => {
                self.launch_rpg_maker_game(game, game_path, container_root, &options)
                    .await?
            }
            EngineType::RpgMakerMV | EngineType::RpgMakerMZ => {
                self.launch_nwjs_game(game, game_path, container_root, nwjs_runtime_dir, &options)
                    .await?
            }
            EngineType::RenPy => {
                self.launch_renpy_game(game, game_path, container_root, &options)
                    .await?
            }
            EngineType::Other => {
                self.launch_other_game(game, game_path, container_root, &options)
                    .await?
            }
        };

        let pid = child.id();

        Ok(LaunchResult { pid })
    }

    /// 启动 RPG Maker (VX/VX Ace) 游戏
    async fn launch_rpg_maker_game(
        &self,
        game: &Game,
        game_path: &Path,
        container_root: &Path,
        options: &LaunchOptions,
    ) -> Result<Child, String> {
        // 查找游戏可执行文件
        let exe_path = self.find_rpg_maker_executable(game_path, options.entry_path.as_deref())?;

        // 设置工作目录为游戏目录
        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(game_path);

        self.apply_home_sandbox(&mut cmd, container_root, &game.profile_key, options);
        self.apply_args(&mut cmd, options);

        // 启动进程
        let child = cmd.spawn().map_err(|e| format!("启动游戏失败: {}", e))?;

        Ok(child)
    }

    /// 启动 NWjs 游戏
    async fn launch_nwjs_game(
        &self,
        game: &Game,
        game_path: &Path,
        container_root: &Path,
        nwjs_runtime_dir: Option<&Path>,
        options: &LaunchOptions,
    ) -> Result<Child, String> {
        // 查找nw可执行文件
        let nw_path = self.find_nwjs_executable(game_path, nwjs_runtime_dir)?;

        let mut cmd = Command::new(&nw_path);
        cmd.current_dir(game_path);

        self.apply_nwjs_sandbox(&mut cmd, container_root, &game.profile_key, options);
        self.apply_args(&mut cmd, options);

        let app_path = self.resolve_nwjs_app_path(game_path, options.entry_path.as_deref());
        cmd.arg(app_path);

        let child = cmd
            .spawn()
            .map_err(|e| format!("启动NWjs游戏失败: {}", e))?;

        Ok(child)
    }

    /// 启动 RenPy 游戏
    async fn launch_renpy_game(
        &self,
        game: &Game,
        game_path: &Path,
        container_root: &Path,
        options: &LaunchOptions,
    ) -> Result<Child, String> {
        // 查找RenPy可执行文件
        let exe_path = self.find_renpy_executable(game_path, options.entry_path.as_deref())?;

        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(game_path);

        self.apply_home_sandbox(&mut cmd, container_root, &game.profile_key, options);
        self.apply_args(&mut cmd, options);

        let child = cmd
            .spawn()
            .map_err(|e| format!("启动RenPy游戏失败: {}", e))?;

        Ok(child)
    }

    async fn launch_other_game(
        &self,
        game: &Game,
        game_path: &Path,
        container_root: &Path,
        options: &LaunchOptions,
    ) -> Result<Child, String> {
        let entry_path = self
            .resolve_entry_path(game_path, options.entry_path.as_deref())
            .ok_or_else(|| "未配置入口文件".to_string())?;

        let mut cmd = Command::new(&entry_path);
        cmd.current_dir(game_path);

        self.apply_home_sandbox(&mut cmd, container_root, &game.profile_key, options);
        self.apply_args(&mut cmd, options);

        let child = cmd.spawn().map_err(|e| format!("启动游戏失败: {}", e))?;

        Ok(child)
    }

    /// 查找 RPG Maker 可执行文件
    fn find_rpg_maker_executable(
        &self,
        game_path: &Path,
        entry_path: Option<&str>,
    ) -> Result<PathBuf, String> {
        if let Some(path) = self.resolve_entry_path(game_path, entry_path) {
            return Ok(path);
        }

        // 尝试查找Game.exe或类似文件
        let candidates = ["Game", "Game.exe", "RPG_RT", "RPG_RT.exe"];

        for candidate in &candidates {
            let exe_path = game_path.join(candidate);
            if exe_path.exists() {
                return Ok(exe_path);
            }
        }

        Err("未找到RPG Maker可执行文件".to_string())
    }

    /// 查找 NWjs 可执行文件
    fn find_nwjs_executable(
        &self,
        game_path: &Path,
        nwjs_runtime_dir: Option<&Path>,
    ) -> Result<PathBuf, String> {
        if let Some(runtime_dir) = nwjs_runtime_dir {
            if let Some(exe) = self.find_nwjs_in_dir(runtime_dir) {
                return Ok(exe);
            }
        }

        if let Some(exe) = self.find_nwjs_in_dir(game_path) {
            return Ok(exe);
        }

        // 尝试使用系统安装的nw
        if let Ok(output) = Command::new("which").arg("nw").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path_str));
            }
        }

        Err("未找到NWjs可执行文件".to_string())
    }

    fn find_nwjs_in_dir(&self, dir: &Path) -> Option<PathBuf> {
        let candidates = ["nw", "nw.exe", "nwjs", "nwjs.exe", "Game", "Game.exe"];

        for candidate in &candidates {
            let exe_path = dir.join(candidate);
            if exe_path.exists() {
                return Some(exe_path);
            }
        }

        let mac_app = dir.join("nwjs.app/Contents/MacOS/nwjs");
        if mac_app.exists() {
            return Some(mac_app);
        }

        None
    }

    fn resolve_launch_options(&self, config: Option<&GameConfig>) -> LaunchOptions {
        if let Some(config) = config {
            let entry_path = config.entry_path.trim();
            LaunchOptions {
                entry_path: if entry_path.is_empty() {
                    None
                } else {
                    Some(entry_path.to_string())
                },
                args: config.args.clone(),
                sandbox_home: config.sandbox_home,
            }
        } else {
            LaunchOptions {
                entry_path: None,
                args: Vec::new(),
                sandbox_home: true,
            }
        }
    }

    fn resolve_entry_path(&self, game_path: &Path, entry_path: Option<&str>) -> Option<PathBuf> {
        let entry = entry_path?.trim();
        if entry.is_empty() {
            return None;
        }

        let candidate = PathBuf::from(entry);
        if candidate.is_absolute() {
            if candidate.exists() {
                return Some(candidate);
            }
            return None;
        }

        let joined = game_path.join(entry);
        if joined.exists() {
            return Some(joined);
        }

        None
    }

    fn resolve_nwjs_app_path(&self, game_path: &Path, entry_path: Option<&str>) -> PathBuf {
        if let Some(path) = self.resolve_entry_path(game_path, entry_path) {
            return path;
        }

        if game_path.join("package.json").exists() {
            return game_path.to_path_buf();
        }

        let www = game_path.join("www");
        if www.join("package.json").exists() {
            return www;
        }

        game_path.to_path_buf()
    }

    fn find_root_executable(&self, game_path: &Path) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            return self.find_windows_executable(game_path);
        }

        #[cfg(target_os = "macos")]
        {
            return self.find_macos_executable(game_path);
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            return self.find_unix_executable(game_path);
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
        {
            let _ = game_path;
            None
        }
    }

    fn apply_home_sandbox(
        &self,
        cmd: &mut Command,
        container_root: &Path,
        profile_key: &str,
        options: &LaunchOptions,
    ) {
        if !options.sandbox_home {
            return;
        }

        let user_data_dir = self
            .file_service
            .game_user_data_dir(container_root, profile_key);
        cmd.env("HOME", user_data_dir);
    }

    fn apply_nwjs_sandbox(
        &self,
        cmd: &mut Command,
        container_root: &Path,
        profile_key: &str,
        options: &LaunchOptions,
    ) {
        if !options.sandbox_home {
            return;
        }

        let user_data_dir = self
            .file_service
            .game_user_data_dir(container_root, profile_key);
        cmd.arg(format!("--user-data-dir={}", user_data_dir.display()));

        let crash_dir = self
            .file_service
            .game_crash_dir(container_root, profile_key);
        cmd.arg(format!("--crash-dumps-dir={}", crash_dir.display()));
        cmd.env("BREAKPAD_DUMP_LOCATION", crash_dir);
    }

    fn apply_args(&self, cmd: &mut Command, options: &LaunchOptions) {
        if !options.args.is_empty() {
            cmd.args(&options.args);
        }
    }

    #[cfg(target_os = "windows")]
    fn find_windows_executable(&self, game_path: &Path) -> Option<PathBuf> {
        self.find_executable_by_extension(game_path, &["exe", "bat", "cmd"])
    }

    #[cfg(target_os = "macos")]
    fn find_macos_executable(&self, game_path: &Path) -> Option<PathBuf> {
        if let Ok(entries) = std::fs::read_dir(game_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("app") {
                    continue;
                }

                let macos_dir = path.join("Contents").join("MacOS");
                if let Ok(binaries) = std::fs::read_dir(&macos_dir) {
                    for bin in binaries.flatten() {
                        let bin_path = bin.path();
                        if bin_path.is_file() {
                            return Some(bin_path);
                        }
                    }
                }
            }
        }

        None
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn find_unix_executable(&self, game_path: &Path) -> Option<PathBuf> {
        if let Some(path) = self.find_executable_by_extension(game_path, &["sh"]) {
            if self.is_executable(&path) {
                return Some(path);
            }
        }

        if let Some(path) = self.find_executable_with_exec_bit(game_path) {
            return Some(path);
        }

        if let Some(path) = self.find_executable_by_extension(game_path, &["py"]) {
            if self.is_executable(&path) {
                return Some(path);
            }
        }

        None
    }

    fn find_executable_by_extension(&self, game_path: &Path, exts: &[&str]) -> Option<PathBuf> {
        let dir_name = game_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mut fallback = None;

        if let Ok(entries) = std::fs::read_dir(game_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !exts.iter().any(|e| *e == ext) {
                    continue;
                }
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !dir_name.is_empty() && stem == dir_name {
                    return Some(path);
                }
                if fallback.is_none() {
                    fallback = Some(path);
                }
            }
        }

        fallback
    }

    #[cfg(unix)]
    fn find_executable_with_exec_bit(&self, game_path: &Path) -> Option<PathBuf> {
        let dir_name = game_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mut fallback = None;

        if let Ok(entries) = std::fs::read_dir(game_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                if !self.is_executable(&path) {
                    continue;
                }
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !dir_name.is_empty() && stem == dir_name {
                    return Some(path);
                }
                if fallback.is_none() {
                    fallback = Some(path);
                }
            }
        }

        fallback
    }

    #[cfg(unix)]
    fn is_executable(&self, path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
        false
    }

    /// 查找 RenPy 可执行文件
    fn find_renpy_executable(
        &self,
        game_path: &Path,
        entry_path: Option<&str>,
    ) -> Result<PathBuf, String> {
        if let Some(path) = self.resolve_entry_path(game_path, entry_path) {
            return Ok(path);
        }

        self.find_root_executable(game_path)
            .ok_or_else(|| "未找到RenPy可执行文件".to_string())
    }
}

impl Default for LauncherService {
    fn default() -> Self {
        Self::new()
    }
}
