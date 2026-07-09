//! 封面自动解析：从游戏目录按引擎类型提取封面图标，支持本地缓存读取和回退机制。

use super::game_executable::resolve_exe_candidate_for_icon;
use crate::commands::state::{ConfigCache, cached_read_config};
use crate::db::schema::Game;
use crate::models::{EngineType, GameDto};
use crate::services::FileService;
use std::path::{Path, PathBuf};

/// 按引擎类型从游戏目录提取封面：优先查找图标目录图片，其次提取 exe 图标，最后回退到封面图片。
pub(crate) fn resolve_cover_for_game(
    file_service: &FileService,
    root: &Path,
    profile_key: &str,
    engine_type: &str,
    game_dir: &Path,
    entry_exe: Option<&Path>,
) -> Option<PathBuf> {
    let engine = EngineType::from_str(engine_type);
    let exe_candidate = resolve_exe_candidate_for_icon(engine.clone(), game_dir, entry_exe);

    let save_image = |path: &Path| {
        file_service
            .save_cover_to_profile(root, profile_key, path)
            .ok()
    };
    let save_exe_icon =
        |path: &Path| file_service.save_exe_icon_to_profile(root, profile_key, path);

    match engine {
        EngineType::RpgMakerVX
        | EngineType::RpgMakerVXAce
        | EngineType::RpgMakerMV
        | EngineType::RpgMakerMZ => {
            if let Some(icon) = file_service.find_icon_dir_image(game_dir) {
                if let Some(saved) = save_image(&icon) {
                    return Some(saved);
                }
            }
            if let Some(exe) = exe_candidate.as_deref() {
                if let Some(saved) = save_exe_icon(exe) {
                    return Some(saved);
                }
            }
            if let Some(cover) = file_service.find_cover_image(game_dir) {
                return save_image(&cover);
            }
        }
        EngineType::RenPy
        | EngineType::Unity
        | EngineType::Godot
        | EngineType::Html
        | EngineType::Other => {
            if let Some(exe) = exe_candidate.as_deref() {
                if let Some(saved) = save_exe_icon(exe) {
                    return Some(saved);
                }
            }
            if let Some(cover) = file_service.find_cover_image(game_dir) {
                return save_image(&cover);
            }
        }
    }

    None
}

/// 查找已有封面：先检查数据库记录的路径，再读取配置文件的 cover_file，最后遍历 profile 目录。
pub(crate) fn resolve_existing_cover(
    file_service: &FileService,
    root: &Path,
    game: &Game,
) -> Option<PathBuf> {
    if let Some(current) = game.cover_path.as_deref() {
        let path = PathBuf::from(current);
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }

    let config_path = file_service.game_config_path(root, &game.profile_key);
    if config_path.exists()
        && let Ok(config) = file_service.read_game_config(&config_path)
        && let Some(cover_file) = config.cover_file
    {
        let profile_dir = file_service.game_profile_dir(root, &game.profile_key);
        let cover_path = if Path::new(&cover_file).is_absolute() {
            PathBuf::from(&cover_file)
        } else {
            profile_dir.join(&cover_file)
        };
        if cover_path.exists() && cover_path.is_file() {
            return Some(cover_path);
        }
    }

    let profile_dir = file_service.game_profile_dir(root, &game.profile_key);
    if !profile_dir.exists() || !profile_dir.is_dir() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(&profile_dir) {
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
            if matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "ico"
            ) {
                return Some(path);
            }
        }
    }

    None
}

/// 将配置中的入口路径解析为实际文件路径，支持绝对路径和相对于游戏目录的路径。
pub(crate) fn resolve_entry_path_for_cover(game_path: &Path, entry_path: &str) -> Option<PathBuf> {
    let entry = entry_path.trim();
    if entry.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(entry);
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        game_path.join(entry)
    };

    if resolved.exists() && resolved.is_file() {
        Some(resolved)
    } else {
        None
    }
}

/// 从游戏配置缓存中补充封面路径到 GameDto，若当前封面失效则尝试读取配置中的 cover_file。
pub(crate) fn fill_cover_from_config(
    cache: &ConfigCache,
    file_service: &FileService,
    root: &Path,
    game: &Game,
    mut dto: GameDto,
) -> GameDto {
    if let Some(path) = dto.cover_path.as_deref() {
        if Path::new(path).exists() {
            return dto;
        }
        dto.cover_path = None;
    }

    let config_path = file_service.game_config_path(root, &game.profile_key);
    let config = cached_read_config(cache, file_service, &config_path, &game.profile_key);
    let Some(config) = config else { return dto };
    let cover_file = config.cover_file.unwrap_or_default();
    if cover_file.trim().is_empty() {
        return dto;
    }

    let profile_dir = file_service.game_profile_dir(root, &game.profile_key);
    let cover_path = if Path::new(&cover_file).is_absolute() {
        PathBuf::from(&cover_file)
    } else {
        profile_dir.join(&cover_file)
    };

    if cover_path.exists() {
        dto.cover_path = Some(cover_path.to_string_lossy().to_string());
    }

    dto
}
