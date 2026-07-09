use crate::model::EngineType;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_exe_candidate_for_icon(
    engine: EngineType,
    game_dir: &Path,
    entry_exe: Option<&Path>,
) -> Option<PathBuf> {
    let entry = entry_exe
        .filter(|p| p.exists() && p.is_file())
        .map(|p| p.to_path_buf());

    match engine {
        EngineType::RenPy => {
            if let Some(entry_path) = entry.as_deref() {
                if let Some(path) = resolve_renpy_icon_exe(entry_path, game_dir) {
                    return Some(path);
                }
            }
            find_executable_for_icon(EngineType::RenPy, game_dir)
        }
        _ => entry.or_else(|| find_executable_for_icon(engine, game_dir)),
    }
}

fn resolve_renpy_icon_exe(entry_exe: &Path, game_dir: &Path) -> Option<PathBuf> {
    let ext = entry_exe
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "exe" {
        return Some(entry_exe.to_path_buf());
    }

    if ext == "sh" {
        let sibling_exe = entry_exe.with_extension("exe");
        if sibling_exe.exists() && sibling_exe.is_file() {
            return Some(sibling_exe);
        }
    }

    let sibling_dir = entry_exe.parent().unwrap_or(game_dir);
    find_root_windows_exe(sibling_dir, &["renpy", "python"])
        .or_else(|| find_root_windows_exe(game_dir, &["renpy", "python"]))
}

fn find_executable_for_icon(engine: EngineType, game_dir: &Path) -> Option<PathBuf> {
    match engine {
        EngineType::RpgMakerVX
        | EngineType::RpgMakerVXAce
        | EngineType::RpgMakerMV
        | EngineType::RpgMakerMZ => find_executable_by_candidates(
            game_dir,
            &[
                "Game.exe",
                "Game",
                "RPG_RT.exe",
                "RPG_RT",
                "nw.exe",
                "nwjs.exe",
            ],
        )
        .or_else(|| find_root_windows_exe(game_dir, &[])),
        EngineType::RenPy => find_root_windows_exe(game_dir, &["renpy", "python"]),
        EngineType::Unity => find_unity_executable(game_dir),
        EngineType::Godot => find_godot_executable(game_dir),
        EngineType::Html | EngineType::Other => find_root_windows_exe(game_dir, &[]),
    }
}

fn find_unity_executable(game_dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_Data") && entry.path().is_dir() {
                let exe_name = name.trim_end_matches("_Data");
                let exe_path = game_dir.join(format!("{}.exe", exe_name));
                if exe_path.exists() {
                    return Some(exe_path);
                }
                let linux_exe = game_dir.join(exe_name);
                if linux_exe.exists() && linux_exe.is_file() {
                    return Some(linux_exe);
                }
            }
        }
    }
    find_root_windows_exe(game_dir, &["UnityCrashHandler", "CrashHandler"])
}

fn find_godot_executable(game_dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".pck") {
                let exe_name = name.trim_end_matches(".pck");
                let exe_path = game_dir.join(format!("{}.exe", exe_name));
                if exe_path.exists() {
                    return Some(exe_path);
                }
                let linux_exe = game_dir.join(exe_name);
                if linux_exe.exists() && linux_exe.is_file() {
                    return Some(linux_exe);
                }
            }
        }
    }
    find_root_windows_exe(game_dir, &[])
}

pub(crate) fn find_renpy_launch_script(game_dir: &Path) -> Option<PathBuf> {
    let dir_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut fallback: Option<PathBuf> = None;
    if let Ok(entries) = std::fs::read_dir(game_dir) {
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
            if ext != "sh" {
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
            if stem == "renpy" {
                continue;
            }
            if fallback.is_none() {
                fallback = Some(path);
            }
        }
    }

    if fallback.is_some() {
        return fallback;
    }

    let renpy_sh = game_dir.join("renpy.sh");
    if renpy_sh.exists() && renpy_sh.is_file() {
        return Some(renpy_sh);
    }

    None
}

fn find_executable_by_candidates(game_dir: &Path, candidates: &[&str]) -> Option<PathBuf> {
    for candidate in candidates {
        let path = game_dir.join(candidate);
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }
    None
}

fn find_root_windows_exe(game_dir: &Path, excluded: &[&str]) -> Option<PathBuf> {
    let dir_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut fallback = None;

    if let Ok(entries) = std::fs::read_dir(game_dir) {
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
            if ext != "exe" {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if excluded.iter().any(|e| *e == stem) {
                continue;
            }
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
