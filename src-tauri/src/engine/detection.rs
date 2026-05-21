use std::path::Path;

use super::context::DetectionContext;
use super::profile::DetectionRuleDef;

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
        "has_extension" => Ok(Box::new(HasExtensionRule {
            ext: def.ext.clone(),
            weight: def.weight,
        })),
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

pub fn find_executable(
    game_dir: &Path,
    patterns: &[String],
    exclude_patterns: &[String],
) -> Option<std::path::PathBuf> {
    for pattern in patterns {
        let candidate = game_dir.join(pattern);
        if candidate.is_file() {
            return Some(candidate);
        }
        if let Ok(entries) = std::fs::read_dir(game_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let name_lower = name.to_lowercase();
                let pat_lower = pattern.to_lowercase();

                if name_lower == pat_lower
                    || name_lower == format!("{}.exe", pat_lower)
                    || crate::engine::context::simple_glob_match(pattern, &name)
                {
                    if !is_excluded(&name_lower, exclude_patterns) {
                        return Some(entry.path());
                    }
                }
            }
        }
    }
    None
}

fn is_excluded(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        crate::engine::context::simple_glob_match(p, name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::context::FsDetectionContext;

    #[test]
    fn file_exists_rule_matches() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), b"").unwrap();

        let ctx = FsDetectionContext::new(dir.path().to_path_buf());
        let rule = FileExistsRule {
            path: "test.txt".into(),
            weight: 3,
        };

        assert!(rule.evaluate(&ctx));
        assert_eq!(rule.weight(), 3);
        assert_eq!(rule.rule_type(), "file_exists");
    }

    #[test]
    fn file_exists_rule_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = FsDetectionContext::new(dir.path().to_path_buf());
        let rule = FileExistsRule {
            path: "missing.txt".into(),
            weight: 3,
        };

        assert!(!rule.evaluate(&ctx));
    }

    #[test]
    fn score_game_accumulates_weights() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"").unwrap();

        let ctx = FsDetectionContext::new(dir.path().to_path_buf());
        let rules: Vec<Box<dyn DetectionRule>> = vec![
            Box::new(FileExistsRule {
                path: "a.txt".into(),
                weight: 2,
            }),
            Box::new(FileExistsRule {
                path: "b.txt".into(),
                weight: 3,
            }),
            Box::new(FileExistsRule {
                path: "c.txt".into(),
                weight: 5,
            }),
        ];

        assert_eq!(score_game(&rules, &ctx), 5);
    }
}
