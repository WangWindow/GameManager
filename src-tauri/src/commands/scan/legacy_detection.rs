//! 传统引擎检测：基于文件系统特征（目录结构、关键文件存在性）对游戏目录进行引擎类型评分识别，覆盖 RPG Maker VX/VXAce/MV/MZ、RenPy、Unity 和 Godot。

use std::path::Path;

/// 基于文件特征检测目录的游戏引擎类型，返回最高评分引擎的标识字符串。
pub(crate) fn detect_engine_type(path: &Path) -> Option<String> {
    detect_engine_with_score(path).map(|(engine, _)| engine)
}

fn detect_engine_with_score(path: &Path) -> Option<(String, i32)> {
    let candidates = [
        ("rpgmakermz", score_rpg_maker_mz(path), 1),
        ("rpgmakermv", score_rpg_maker_mv(path), 2),
        ("renpy", score_renpy(path), 0),
        ("rpgmakervxace", score_rpg_maker_vxace(path), 3),
        ("rpgmakervx", score_rpg_maker_vx(path), 4),
        ("unity", score_unity(path), 5),
        ("godot", score_godot(path), 6),
    ];

    let mut best: Option<(&str, i32, i32)> = None;
    for (engine, score, priority) in candidates {
        if score < min_engine_score(engine) {
            continue;
        }
        match best {
            None => best = Some((engine, score, priority)),
            Some((_, best_score, best_priority)) => {
                if score > best_score || (score == best_score && priority < best_priority) {
                    best = Some((engine, score, priority));
                }
            }
        }
    }

    best.map(|(engine, score, _)| (engine.to_string(), (score * 16).min(100)))
}

fn min_engine_score(engine: &str) -> i32 {
    match engine {
        "rpgmakermz" | "rpgmakermv" => 4,
        "renpy" => 4,
        "rpgmakervxace" | "rpgmakervx" => 5,
        "unity" => 4,
        "godot" => 4,
        _ => 0,
    }
}

fn score_rpg_maker_mz(path: &Path) -> i32 {
    let base = path;
    let www = path.join("www");
    let mut score = 0;
    if has_mz_core(base) || has_mz_core(&www) {
        score += 3;
    }
    if has_rpg_data(base) || has_rpg_data(&www) {
        score += 2;
    }
    if has_package_json(base) || has_package_json(&www) {
        score += 1;
    }
    score
}

fn score_rpg_maker_mv(path: &Path) -> i32 {
    let base = path;
    let www = path.join("www");
    let mut score = 0;
    if has_mv_core(base) || has_mv_core(&www) {
        score += 3;
    }
    if has_rpg_data(base) || has_rpg_data(&www) {
        score += 2;
    }
    if has_package_json(base) || has_package_json(&www) {
        score += 1;
    }
    score
}

fn score_renpy(path: &Path) -> i32 {
    let mut score = 0;
    if path.join("renpy").is_dir()
        || path.join("renpy.sh").exists()
        || path.join("renpy.exe").exists()
    {
        score += 3;
    }

    let game_dir = path.join("game");
    if game_dir.is_dir() {
        score += 1;
        if has_renpy_scripts(&game_dir) {
            score += 3;
        }
        if has_renpy_marker_files(&game_dir) {
            score += 1;
        }
    }

    if has_renpy_lib(path) {
        score += 1;
    }

    score
}

fn score_rpg_maker_vxace(path: &Path) -> i32 {
    let mut score = 0;
    if has_vx_executable(path) {
        score += 2;
    }
    if has_rgss_dll(path, "RGSS3") {
        score += 3;
    }
    if path.join("Game.ini").exists() {
        score += 1;
    }
    score
}

fn score_rpg_maker_vx(path: &Path) -> i32 {
    let mut score = 0;
    if has_vx_executable(path) {
        score += 2;
    }
    if has_rgss_dll(path, "RGSS2") || has_rgss_dll(path, "RGSS1") {
        score += 3;
    }
    if path.join("Game.ini").exists() {
        score += 1;
    }
    score
}

fn score_unity(path: &Path) -> i32 {
    let mut score = 0;
    if path.join("UnityPlayer.dll").exists() {
        score += 3;
    }
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_Data") && entry.path().is_dir() {
                let data_dir = entry.path();
                score += 2;
                if data_dir.join("Managed").is_dir() {
                    score += 1;
                }
                if data_dir.join("globalgamemanagers").exists()
                    || data_dir.join("mainData").exists()
                {
                    score += 1;
                }
                break;
            }
        }
    }
    if path.join("MonoBleedingEdge").is_dir() {
        score += 1;
    }
    if path.join("GameAssembly.dll").exists() {
        score += 2;
    }
    score
}

fn score_godot(path: &Path) -> i32 {
    let mut score = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".pck") {
                score += 3;
                break;
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("godot") && (name.ends_with(".dll") || name.ends_with(".so")) {
                score += 2;
                break;
            }
        }
    }
    if path.join(".import").is_dir() || path.join(".godot").is_dir() {
        score += 1;
    }
    if path.join("project.godot").exists() {
        score += 2;
    }
    score
}

fn has_mz_core(base: &Path) -> bool {
    let js = base.join("js");
    js.join("rmmz_core.js").exists() || js.join("rmmz_managers.js").exists()
}

fn has_mv_core(base: &Path) -> bool {
    let js = base.join("js");
    js.join("rpg_core.js").exists() || js.join("rpg_managers.js").exists()
}

fn has_rpg_data(base: &Path) -> bool {
    base.join("data").join("System.json").exists()
}

fn has_package_json(base: &Path) -> bool {
    base.join("package.json").exists()
}

fn has_vx_executable(path: &Path) -> bool {
    ["Game.exe", "Game", "RPG_RT.exe", "RPG_RT"]
        .iter()
        .any(|name| path.join(name).exists())
}

fn has_rgss_dll(path: &Path, prefix: &str) -> bool {
    let prefix = prefix.to_lowercase();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_lowercase();
            if name.starts_with(&prefix) && name.ends_with(".dll") {
                return true;
            }
        }
    }
    false
}

fn has_renpy_marker_files(game_dir: &Path) -> bool {
    let marker_files = ["script.rpy", "options.rpy", "gui.rpy", "screens.rpy"];
    marker_files.iter().any(|name| game_dir.join(name).exists())
}

fn has_renpy_scripts(game_dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(), "rpy" | "rpyc"))
                == Some(true)
            {
                return true;
            }
        }
    }
    false
}

fn has_renpy_lib(path: &Path) -> bool {
    let lib_dir = path.join("lib");
    if !lib_dir.is_dir() {
        return false;
    }

    if let Ok(entries) = std::fs::read_dir(&lib_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.starts_with("py") || name.contains("python") {
                return true;
            }
        }
    }

    false
}
