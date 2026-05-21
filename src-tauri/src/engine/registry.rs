use std::collections::HashMap;
use std::path::Path;

use super::context::DetectionContext;
use super::detection::{build_rule, score_game, DetectionRule};
use super::launch::{build_strategy, LaunchStrategy};
use super::profile::{EngineMetaDto, EngineProfile};

pub struct EngineEntry {
    pub profile: EngineProfile,
    pub rules: Vec<Box<dyn DetectionRule>>,
    pub strategy: Box<dyn LaunchStrategy>,
    pub enabled: bool,
    pub valid: bool,
    pub errors: Vec<String>,
}

pub struct EngineRegistry {
    entries: HashMap<String, EngineEntry>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// 从目录加载所有 .toml 引擎配置。
    ///
    /// `enabled_map` 来自持久化存储，决定每个引擎是否启用（key = engine id）。
    /// 加载失败的引擎记录错误但不阻塞其他引擎。
    pub fn load(
        &mut self,
        config_dir: &Path,
        enabled_map: &HashMap<String, bool>,
    ) -> Vec<String> {
        let mut warnings: Vec<String> = Vec::new();

        let entries = match std::fs::read_dir(config_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warnings.push(format!("无法读取引擎配置目录 {}: {}", config_dir.display(), e));
                return warnings;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            match self.load_one(&path, enabled_map) {
                Ok(()) => {}
                Err(err) => {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    warnings.push(format!("{}: {}", name, err));
                }
            }
        }

        warnings
    }

    fn load_one(
        &mut self,
        path: &Path,
        enabled_map: &HashMap<String, bool>,
    ) -> Result<(), String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;

        let profile: EngineProfile =
            toml::from_str(&content).map_err(|e| format!("TOML 解析失败: {}", e))?;

        let id = profile.meta.id.clone();

        let mut errors: Vec<String> = Vec::new();

        // 验证检测规则
        for rule_def in &profile.detection.rules {
            if let Err(e) = rule_def.validate() {
                errors.push(format!("检测规则 '{}': {}", rule_def.rule_type, e));
            }
        }

        // 验证启动配置
        if let Err(e) = profile.launch.validate() {
            errors.push(format!("启动配置: {}", e));
        }

        let valid = errors.is_empty();

        // 编译检测规则
        let mut rules: Vec<Box<dyn DetectionRule>> = Vec::new();
        if valid {
            for rule_def in &profile.detection.rules {
                match build_rule(rule_def) {
                    Ok(rule) => rules.push(rule),
                    Err(e) => {
                        errors.push(e);
                    }
                }
            }
        }

        // 编译启动策略
        let strategy = if valid && errors.is_empty() {
            match build_strategy(&profile.launch.strategy) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(format!("启动策略: {}", e));
                    // fallback: 用一个会报错的 placeholder
                    build_strategy("native").unwrap()
                }
            }
        } else {
            build_strategy("native").unwrap()
        };

        let final_valid = valid && errors.is_empty();

        // 如果验证失败，只能由用户手动启用
        let enabled = if final_valid {
            enabled_map.get(&id).copied().unwrap_or(true)
        } else {
            enabled_map.get(&id).copied().unwrap_or(false)
        };

        self.entries.insert(
            id,
            EngineEntry {
                profile,
                rules,
                strategy,
                enabled: enabled && final_valid,
                valid: final_valid,
                errors,
            },
        );

        Ok(())
    }

    /// 检测游戏目录匹配哪个引擎。
    ///
    /// 返回 `(engine_id, confidence)`，confidence 为 0-100。
    /// 只检查 `enabled && valid` 的引擎。
    pub fn detect(
        &self,
        ctx: &dyn DetectionContext,
    ) -> Option<(&str, i32)> {
        let mut best: Option<(&str, i32, i32)> = None; // (id, score, priority)

        for entry in self.entries.values() {
            if !entry.enabled || !entry.valid {
                continue;
            }

            let raw_score = score_game(&entry.rules, ctx);
            let min_score = entry.profile.detection.min_score;

            if raw_score < min_score {
                continue;
            }

            let priority = entry.profile.meta.priority;

            match best {
                None => best = Some((entry.profile.meta.id.as_str(), raw_score, priority)),
                Some((_, best_score, best_priority)) => {
                    if raw_score > best_score
                        || (raw_score == best_score && priority < best_priority)
                    {
                        best = Some((entry.profile.meta.id.as_str(), raw_score, priority));
                    }
                }
            }
        }

        best.map(|(id, raw_score, _)| {
            let confidence = (raw_score as f64 / 16.0 * 100.0).min(100.0) as i32;
            (id, confidence.max(1))
        })
    }

    /// 获取前端的引擎摘要列表。
    pub fn list_for_frontend(&self) -> Vec<EngineMetaDto> {
        let mut dtos: Vec<EngineMetaDto> = self
            .entries
            .values()
            .map(|e| EngineMetaDto::from(&e.profile))
            .collect();
        dtos.sort_by_key(|d| d.priority);
        dtos
    }

    /// 获取某个引擎的详细信息（含校验状态）。
    pub fn get_entry(&self, id: &str) -> Option<&EngineEntry> {
        self.entries.get(id)
    }

    /// 设置引擎启用/禁用状态（即时生效）。
    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), String> {
        let entry = self
            .entries
            .get_mut(id)
            .ok_or_else(|| format!("引擎 '{}' 不存在", id))?;

        if !entry.valid && enabled {
            return Err(format!(
                "引擎 '{}' 校验失败，无法启用。错误: {}",
                id,
                entry.errors.join("; ")
            ));
        }

        entry.enabled = enabled;
        Ok(())
    }

    /// 检查是否有至少一个启用的引擎。
    pub fn has_enabled_engine(&self) -> bool {
        self.entries.values().any(|e| e.enabled && e.valid)
    }

    pub fn engine_count(&self) -> usize {
        self.entries.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.entries.values().filter(|e| e.enabled && e.valid).count()
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::context::FsDetectionContext;

    fn make_test_registry() -> EngineRegistry {
        let mut registry = EngineRegistry::new();

        let toml_str = r#"
[meta]
id = "test-engine"
name = "Test Engine"
category = "test"

[detection]
min_score = 2

[[detection.rules]]
type = "file_exists"
path = "marker.txt"
weight = 3

[launch]
strategy = "native"
entry_patterns = ["game"]
"#;

        let profile: EngineProfile = toml::from_str(toml_str).unwrap();
        let rules: Vec<Box<dyn DetectionRule>> =
            profile.detection.rules.iter()
                .map(|d| build_rule(d).unwrap())
                .collect();
        let strategy = build_strategy("native").unwrap();

        registry.entries.insert(
            "test-engine".into(),
            EngineEntry {
                profile,
                rules,
                strategy,
                enabled: true,
                valid: true,
                errors: vec![],
            },
        );

        registry
    }

    #[test]
    fn detect_matching_game() {
        let registry = make_test_registry();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("marker.txt"), b"").unwrap();

        let ctx = FsDetectionContext::new(dir.path().to_path_buf());
        let result = registry.detect(&ctx);

        assert!(result.is_some());
        let (id, confidence) = result.unwrap();
        assert_eq!(id, "test-engine");
        assert!(confidence > 0);
    }

    #[test]
    fn detect_no_match() {
        let registry = make_test_registry();
        let dir = tempfile::tempdir().unwrap();

        let ctx = FsDetectionContext::new(dir.path().to_path_buf());
        let result = registry.detect(&ctx);

        assert!(result.is_none());
    }

    #[test]
    fn disabled_engine_not_detected() {
        let mut registry = make_test_registry();
        registry.set_enabled("test-engine", false).unwrap();

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("marker.txt"), b"").unwrap();

        let ctx = FsDetectionContext::new(dir.path().to_path_buf());
        assert!(registry.detect(&ctx).is_none());
    }

    #[test]
    fn cannot_enable_invalid_engine() {
        let mut registry = make_test_registry();
        registry.entries.get_mut("test-engine").unwrap().valid = false;
        registry.entries.get_mut("test-engine").unwrap().enabled = false;
        registry.entries.get_mut("test-engine").unwrap().errors = vec!["bad config".into()];

        let result = registry.set_enabled("test-engine", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("校验失败"));
    }

    #[test]
    fn list_for_frontend_returns_dtos() {
        let registry = make_test_registry();
        let dtos = registry.list_for_frontend();
        assert_eq!(dtos.len(), 1);
        assert_eq!(dtos[0].id, "test-engine");
        assert_eq!(dtos[0].name, "Test Engine");
    }
}
