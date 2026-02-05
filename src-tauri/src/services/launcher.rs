use crate::models::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// 游戏启动服务
pub struct LauncherService {
    file_service: crate::services::FileService,
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
    ) -> Result<LaunchResult, String> {
        // 检查游戏路径是否存在
        let game_path = Path::new(&game.path);
        if !game_path.exists() {
            return Err(format!("游戏路径不存在: {}", game.path));
        }

        // 确保游戏目录结构存在
        self.file_service
            .ensure_game_dirs(container_root, &game.profile_key)?;

        // 根据引擎类型启动游戏
        let engine_type = game.get_engine_type();
        let child = match engine_type {
            EngineType::RpgMakerVX | EngineType::RpgMakerVXAce => {
                self.launch_rpg_maker_game(game, game_path, container_root)
                    .await?
            }
            EngineType::RpgMakerMV | EngineType::RpgMakerMZ => {
                self.launch_nwjs_game(game, game_path, container_root, nwjs_runtime_dir)
                    .await?
            }
            EngineType::NWjs => {
                self.launch_nwjs_game(game, game_path, container_root, nwjs_runtime_dir)
                    .await?
            }
            EngineType::RenPy => {
                self.launch_renpy_game(game, game_path, container_root)
                    .await?
            }
            EngineType::Other => {
                return Err("不支持的引擎类型".to_string());
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
    ) -> Result<Child, String> {
        // 查找游戏可执行文件
        let exe_path = self.find_rpg_maker_executable(game_path)?;

        // 设置工作目录为游戏目录
        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(game_path);

        // 如果启用沙盒，设置HOME环境变量
        let user_data_dir = self
            .file_service
            .game_user_data_dir(container_root, &game.profile_key);
        cmd.env("HOME", user_data_dir);

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
    ) -> Result<Child, String> {
        // 查找nw可执行文件
        let nw_path = self.find_nwjs_executable(game_path, nwjs_runtime_dir)?;

        let mut cmd = Command::new(&nw_path);
        cmd.arg(game_path);
        cmd.current_dir(game_path);

        // 设置用户数据目录
        let user_data_dir = self
            .file_service
            .game_user_data_dir(container_root, &game.profile_key);
        cmd.arg(format!("--user-data-dir={}", user_data_dir.display()));

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
    ) -> Result<Child, String> {
        // 查找RenPy可执行文件
        let exe_path = self.find_renpy_executable(game_path)?;

        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(game_path);

        // 设置HOME环境变量用于沙盒
        let user_data_dir = self
            .file_service
            .game_user_data_dir(container_root, &game.profile_key);
        cmd.env("HOME", user_data_dir);

        let child = cmd
            .spawn()
            .map_err(|e| format!("启动RenPy游戏失败: {}", e))?;

        Ok(child)
    }

    /// 查找 RPG Maker 可执行文件
    fn find_rpg_maker_executable(&self, game_path: &Path) -> Result<PathBuf, String> {
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

    /// 查找 RenPy 可执行文件
    fn find_renpy_executable(&self, game_path: &Path) -> Result<PathBuf, String> {
        // 查找RenPy可执行文件（通常以游戏名命名）
        if let Ok(entries) = std::fs::read_dir(game_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // 检查是否为可执行文件（Linux下）
                        if !name.ends_with(".py") && !name.ends_with(".txt") {
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                if let Ok(metadata) = std::fs::metadata(&path) {
                                    let permissions = metadata.permissions();
                                    if permissions.mode() & 0o111 != 0 {
                                        return Ok(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("未找到RenPy可执行文件".to_string())
    }
}

impl Default for LauncherService {
    fn default() -> Self {
        Self::new()
    }
}
