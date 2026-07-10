use std::path::Path;

use super::context::DetectionContext;
use super::profile::DetectionRuleDef;

#[allow(dead_code)]
pub trait DetectionRule: Send + Sync {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool;
    fn weight(&self) -> i32;
    fn rule_type(&self) -> &str;
}

pub struct FileExistsRule {
    path: String,
    weight: i32,
}

impl DetectionRule for FileExistsRule {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool {
        ctx.file_exists(&self.path)
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    fn rule_type(&self) -> &str {
        "file_exists"
    }
}

pub struct DirExistsRule {
    path: String,
    weight: i32,
}

impl DetectionRule for DirExistsRule {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool {
        ctx.dir_exists(&self.path)
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    fn rule_type(&self) -> &str {
        "dir_exists"
    }
}

pub struct GlobMatchRule {
    pattern: String,
    weight: i32,
}

pub struct RecursiveGlobMatchRule {
    pattern: String,
    weight: i32,
}

impl DetectionRule for RecursiveGlobMatchRule {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool {
        ctx.glob_match_recursive(&self.pattern, 3)
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    fn rule_type(&self) -> &str {
        "glob_match_recursive"
    }
}

impl DetectionRule for GlobMatchRule {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool {
        ctx.glob_match(&self.pattern)
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    fn rule_type(&self) -> &str {
        "glob_match"
    }
}

pub struct HasExtensionRule {
    ext: String,
    weight: i32,
}

pub struct HasNativeExecutableRule {
    weight: i32,
}

impl DetectionRule for HasNativeExecutableRule {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool {
        ctx.has_native_executable()
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    fn rule_type(&self) -> &str {
        "has_native_executable"
    }
}

impl DetectionRule for HasExtensionRule {
    fn evaluate(&self, ctx: &dyn DetectionContext) -> bool {
        ctx.has_extension(&self.ext)
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    fn rule_type(&self) -> &str {
        "has_extension"
    }
}

pub fn build_rule(def: &DetectionRuleDef) -> Result<Box<dyn DetectionRule>, String> {
    def.validate()?;

    match def.rule_type.as_str() {
        "file_exists" => Ok(Box::new(FileExistsRule {
            path: def.path.clone(),
            weight: def.weight,
        })),
        "dir_exists" => Ok(Box::new(DirExistsRule {
            path: def.path.clone(),
            weight: def.weight,
        })),
        "glob_match" => Ok(Box::new(GlobMatchRule {
            pattern: def.pattern.clone(),
            weight: def.weight,
        })),
        "glob_match_recursive" => Ok(Box::new(RecursiveGlobMatchRule {
            pattern: def.pattern.clone(),
            weight: def.weight,
        })),
        "has_extension" => Ok(Box::new(HasExtensionRule {
            ext: def.ext.clone(),
            weight: def.weight,
        })),
        "has_native_executable" => Ok(Box::new(HasNativeExecutableRule { weight: def.weight })),
        other => Err(format!("未知的检测规则类型: {}", other)),
    }
}

pub fn score_game(rules: &[Box<dyn DetectionRule>], ctx: &dyn DetectionContext) -> i32 {
    let mut score = 0;
    for rule in rules {
        if rule.evaluate(ctx) {
            score += rule.weight();
        }
    }
    score
}

/// 将命中分数按当前插件可获得的总分归一化，避免规则数量影响置信度。
pub fn confidence_score(rules: &[Box<dyn DetectionRule>], score: i32) -> i32 {
    let maximum = rules.iter().map(|rule| rule.weight().max(0)).sum::<i32>();
    if maximum == 0 {
        return 0;
    }
    ((score as f64 / maximum as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as i32
}

pub fn find_executable(
    game_dir: &Path,
    patterns: &[String],
    exclude_patterns: &[String],
) -> Option<std::path::PathBuf> {
    let mut direct_files = std::fs::read_dir(game_dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    direct_files.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    let mut recursive_files: Option<Vec<std::path::PathBuf>> = None;

    for pattern in patterns {
        if pattern == "@native" {
            if let Some(path) = direct_files.iter().find(|path| is_native_executable(path)) {
                return Some(path.clone());
            }
            continue;
        }
        let candidate = game_dir.join(pattern);
        let candidate_name = candidate
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();
        if candidate.is_file() && !is_excluded(&candidate_name, exclude_patterns) {
            return Some(candidate);
        }

        let is_nested_pattern = pattern.contains('/') || pattern.contains('\\');
        let entries = if is_nested_pattern {
            recursive_files.get_or_insert_with(|| {
                let mut files = Vec::new();
                collect_files(game_dir, 4, &mut files);
                files.sort_by_key(|path: &std::path::PathBuf| {
                    path.strip_prefix(game_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/")
                        .to_lowercase()
                });
                files
            })
        } else {
            &direct_files
        };

        for entry in entries {
            let name = entry
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_string();
            let name_lower = name.to_lowercase();
            let pat_lower = pattern.to_lowercase();
            let relative = entry
                .strip_prefix(game_dir)
                .unwrap_or(entry)
                .to_string_lossy()
                .replace('\\', "/");

            if name_lower == pat_lower
                || name_lower == format!("{}.exe", pat_lower)
                || crate::engines::context::simple_glob_match(pattern, &name)
                || (is_nested_pattern
                    && crate::engines::context::simple_glob_match(pattern, &relative))
            {
                if !is_excluded(&name_lower, exclude_patterns) {
                    return Some(entry.clone());
                }
            }
        }
    }
    None
}

fn is_native_executable(path: &Path) -> bool {
    crate::utils::path::is_linux_native_executable(path)
}

/*
 * 普通文件名规则只检查游戏根目录。只有配置中明确包含路径分隔符时，
 * 才会调用该递归收集函数，避免每个游戏都遍历完整资源目录。
 */
fn collect_files(dir: &Path, depth: u32, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            out.push(path);
        } else if depth > 0 && path.is_dir() {
            collect_files(&path, depth - 1, out);
        }
    }
}

fn is_excluded(name: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|p| crate::engines::context::simple_glob_match(p, name))
}
