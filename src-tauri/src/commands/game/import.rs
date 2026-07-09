use super::cover::update_game_cover;
use super::game::{default_game_config, is_nwjs_runtime_dir, normalize_path};
use crate::commands::state::AppState;
use crate::models::{AddGameInput, ImportGameInput, SETTING_BOTTLES_ENABLED};
use crate::services::FileService;
use std::path::Path;
use tauri::State;

/// 导入游戏目录
#[tauri::command]
pub async fn import_game_dir(
    input: ImportGameInput,
    state: State<'_, AppState>,
) -> Result<crate::models::GameDto, String> {
    let service = state.game_service.lock().await;

    let executable_path = normalize_path(Path::new(&input.executable_path));
    let engine_type = input.engine_type;

    let exe_path = Path::new(&executable_path);
    if !exe_path.exists() || !exe_path.is_file() {
        return Err("可执行文件不存在".to_string());
    }

    let game_dir = exe_path
        .parent()
        .ok_or_else(|| "无法解析游戏目录".to_string())?;

    if is_nwjs_runtime_dir(game_dir) {
        return Err("检测到 NW.js 运行器目录，无法作为游戏导入".to_string());
    }

    let title = derive_game_title(exe_path, game_dir);

    let input = AddGameInput {
        title: Some(title),
        engine_type: engine_type.clone(),
        path: normalize_path(game_dir),
        game_type: None,
        detection_confidence: None,
        metadata_json: None,
        runtime_version: None,
    };

    let game = service.add_game(input).await?;

    let root = state.container_root_path().await;

    // 写入默认配置，记录入口文件
    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    if let Err(e) = file_service.ensure_game_dirs(&root, &game.profile_key) {
        return Err(e);
    }
    let mut config = default_game_config(&game);
    let entry_patterns = {
        let registry = state.engine_registry.lock().await;
        registry
            .get_entry(&engine_type)
            .map(|e| e.profile.launch.entry_patterns.clone())
            .unwrap_or_default()
    };
    if entry_patterns.is_empty() {
        if game_dir.join("www").join("package.json").exists() {
            config.entry_path = "www".to_string();
        } else {
            config.entry_path = "".to_string();
        }
    } else {
        config.entry_path = executable_path.clone();
    }
    // 继承全局 Bottles 设置（仅 Windows .exe，Linux 原生不需要）
    {
        let mut db_lock = state.db.lock().await;
        if let Ok(Some(val)) = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_ENABLED).await
        {
            config.use_bottles = val == "1" && executable_path.to_lowercase().ends_with(".exe");
        }
    }
    let _ = file_service.write_game_config(&config_path, &config);

    // 按优先级提取图标/封面
    let entry_exe = Some(exe_path);
    update_game_cover(
        &service,
        &root,
        &game,
        &engine_type,
        game_dir,
        entry_exe,
        false,
    )
    .await;

    Ok(service.to_dto(game))
}

fn derive_game_title(exe_path: &Path, game_dir: &Path) -> String {
    let stem = exe_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim();
    let stem_lower = stem.to_lowercase();
    let invalid_names = ["game", "nw", "nwjs", "rpg_rt"];

    if !stem.is_empty() && !invalid_names.iter().any(|n| *n == stem_lower) {
        return stem.to_string();
    }

    let dir_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim();

    if !dir_name.is_empty() {
        return dir_name.to_string();
    }

    "未命名游戏".to_string()
}
