use crate::engines::EngineRegistry;
use crate::models::GameConfig;
use crate::services::{EngineService, FileService, GameService, LauncherService};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

pub(crate) type ConfigCache = Arc<StdMutex<HashMap<String, GameConfig>>>;

/// 应用状态
pub struct AppState {
    pub game_service: Arc<Mutex<GameService>>,
    pub engine_service: Arc<Mutex<EngineService>>,
    pub launcher_service: Arc<Mutex<LauncherService>>,
    pub db: Arc<Mutex<toasty::Db>>,
    pub container_root: Arc<Mutex<String>>,
    pub engine_registry: Arc<Mutex<EngineRegistry>>,
    pub config_cache: ConfigCache,
}

impl AppState {
    /// 获取容器根目录的规范化绝对路径。
    pub(crate) async fn container_root_path(&self) -> PathBuf {
        let root = self.container_root.lock().await;
        crate::utils::path::canonicalize(Path::new(root.as_str()))
    }
}

/// 带缓存的游戏配置读取。key = profile_key。
/// 缓存生命周期与应用一致，游戏删除时清除对应条目避免泄漏。
pub(crate) fn cached_read_config(
    cache: &ConfigCache,
    file_service: &FileService,
    path: &Path,
    profile_key: &str,
) -> Option<GameConfig> {
    let mut cache = cache.lock().unwrap();
    if let Some(cfg) = cache.get(profile_key) {
        return Some(cfg.clone());
    }
    if let Ok(cfg) = file_service.read_game_config(path) {
        cache.insert(profile_key.to_string(), cfg.clone());
        return Some(cfg);
    }
    None
}

pub(crate) fn cached_write_config(
    cache: &ConfigCache,
    file_service: &FileService,
    path: &Path,
    profile_key: &str,
    config: &GameConfig,
) -> Result<(), String> {
    file_service.write_game_config(path, config)?;
    cache
        .lock()
        .unwrap()
        .insert(profile_key.to_string(), config.clone());
    Ok(())
}

pub(crate) fn cache_remove(cache: &ConfigCache, profile_key: &str) {
    cache.lock().unwrap().remove(profile_key);
}
