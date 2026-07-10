use std::collections::HashMap;
use std::path::Path;

use super::context::DetectionContext;
use super::detection::{DetectionRule, build_rule, confidence_score, score_game};
use super::launch::{LaunchStrategy, build_strategy};
use super::profile::{
    DetectionDetail, EngineDetailDto, EngineMetaDto, EngineProfile, EngineProfileDetailDto,
    LaunchDetail, RuleDetail,
};

pub struct EngineEntry {
    pub profile: EngineProfile,
    pub required_rules: Vec<Box<dyn DetectionRule>>,
    pub optional_rules: Vec<Box<dyn DetectionRule>>,
    pub forbidden_rules: Vec<Box<dyn DetectionRule>>,
    #[allow(dead_code)]
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
    pub fn load(&mut self, config_dir: &Path, enabled_map: &HashMap<String, bool>) -> Vec<String> {
        let mut warnings: Vec<String> = Vec::new();

        let entries = match std::fs::read_dir(config_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warnings.push(format!(
                    "无法读取引擎配置目录 {}: {}",
                    config_dir.display(),
                    e
                ));
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

    fn load_one(&mut self, path: &Path, enabled_map: &HashMap<String, bool>) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;

        let profile: EngineProfile =
            toml::from_str(&content).map_err(|e| format!("TOML 解析失败: {}", e))?;

        let id = profile.meta.id.clone();

        let mut errors: Vec<String> = Vec::new();

        if let Err(e) = profile.detection.validate() {
            errors.push(format!("检测配置: {}", e));
        }
        if path.file_stem().and_then(|name| name.to_str()) != Some(id.as_str()) {
            errors.push(format!("插件 id '{}' 必须与文件名一致", id));
        }

        // 验证所有检测规则组
        for rule_def in profile
            .detection
            .required
            .iter()
            .chain(profile.detection.optional.iter())
            .chain(profile.detection.forbidden.iter())
        {
            if let Err(e) = rule_def.validate() {
                errors.push(format!("检测规则 '{}': {}", rule_def.rule_type, e));
            }
        }

        // 验证启动配置
        if let Err(e) = profile.launch.validate() {
            errors.push(format!("启动配置: {}", e));
        }

        let valid = errors.is_empty();

        let compile_rules = |defs: &[super::profile::DetectionRuleDef]| {
            defs.iter().map(build_rule).collect::<Result<Vec<_>, _>>()
        };
        let required_rules = compile_rules(&profile.detection.required).unwrap_or_else(|e| {
            errors.push(e);
            Vec::new()
        });
        let optional_rules = compile_rules(&profile.detection.optional).unwrap_or_else(|e| {
            errors.push(e);
            Vec::new()
        });
        let forbidden_rules = compile_rules(&profile.detection.forbidden).unwrap_or_else(|e| {
            errors.push(e);
            Vec::new()
        });

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
                required_rules,
                optional_rules,
                forbidden_rules,
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
    pub fn detect(&self, ctx: &dyn DetectionContext) -> Option<(&str, i32)> {
        let mut best_specific: Option<(&str, i32, i32, i32)> = None;
        let mut best_other: Option<(&str, i32, i32, i32)> = None;

        for entry in self.entries.values() {
            if !entry.enabled || !entry.valid {
                continue;
            }

            if entry.required_rules.iter().any(|rule| !rule.evaluate(ctx))
                || entry.forbidden_rules.iter().any(|rule| rule.evaluate(ctx))
            {
                continue;
            }

            let raw_score = score_game(&entry.optional_rules, ctx);
            let min_score = entry.profile.detection.min_score;

            if raw_score < min_score {
                continue;
            }

            let priority = entry.profile.meta.priority;
            let confidence = if entry.required_rules.is_empty() {
                confidence_score(&entry.optional_rules, raw_score).max(1)
            } else if entry.optional_rules.is_empty() {
                100
            } else {
                60 + confidence_score(&entry.optional_rules, raw_score) * 40 / 100
            };

            let target = if entry.profile.meta.id == "other" {
                &mut best_other
            } else {
                &mut best_specific
            };
            match *target {
                None => {
                    *target = Some((
                        entry.profile.meta.id.as_str(),
                        confidence,
                        raw_score,
                        priority,
                    ))
                }
                Some((_, best_confidence, best_score, best_priority)) => {
                    if confidence > best_confidence
                        || (confidence == best_confidence && raw_score > best_score)
                        || (confidence == best_confidence
                            && raw_score == best_score
                            && priority < best_priority)
                    {
                        *target = Some((
                            entry.profile.meta.id.as_str(),
                            confidence,
                            raw_score,
                            priority,
                        ));
                    }
                }
            }
        }

        best_specific
            .or(best_other)
            .map(|(id, confidence, _, _)| (id, confidence))
    }

    /// 获取前端的引擎摘要列表。
    pub fn list_for_frontend(&self) -> Vec<EngineMetaDto> {
        let mut dtos: Vec<EngineMetaDto> = self
            .entries
            .values()
            .map(|e| {
                let mut dto = EngineMetaDto::from(&e.profile);
                dto.enabled = e.enabled;
                dto
            })
            .collect();
        dtos.sort_by_key(|d| d.priority);
        dtos
    }

    /// 插件管理面板用的完整列表（含启用/校验状态）。
    pub fn list_detail_for_frontend(&self) -> Vec<EngineDetailDto> {
        let mut dtos: Vec<EngineDetailDto> = self
            .entries
            .values()
            .map(|e| EngineDetailDto {
                id: e.profile.meta.id.clone(),
                name: e.profile.meta.name.clone(),
                category: e.profile.meta.category.clone(),
                icon: e.profile.meta.icon.clone(),
                description: e.profile.meta.description.clone(),
                enabled: e.enabled,
                valid: e.valid,
                rule_count: e.profile.detection.rule_count(),
                strategy: e.profile.launch.strategy.clone(),
                errors: e.errors.clone(),
            })
            .collect();
        dtos.sort_by_key(|d| d.id.clone());
        dtos
    }

    /// 获取某个引擎的详细信息（含校验状态）。
    pub fn get_entry(&self, id: &str) -> Option<&EngineEntry> {
        self.entries.get(id)
    }

    /// 检测时是否应跳过该引擎。
    pub fn should_skip_scan(&self, id: &str) -> bool {
        self.entries
            .get(id)
            .map(|e| e.profile.meta.skip_scan)
            .unwrap_or(false)
    }

    /// 获取插件完整配置详情。
    pub fn get_profile_detail(&self, id: &str) -> Option<EngineProfileDetailDto> {
        let entry = self.entries.get(id)?;
        let p = &entry.profile;
        Some(EngineProfileDetailDto {
            id: p.meta.id.clone(),
            name: p.meta.name.clone(),
            category: p.meta.category.clone(),
            icon: p.meta.icon.clone(),
            description: p.meta.description.clone(),
            enabled: entry.enabled,
            valid: entry.valid,
            detection: DetectionDetail {
                min_score: p.detection.min_score,
                rules: p
                    .detection
                    .required
                    .iter()
                    .map(|r| ("required", r))
                    .chain(p.detection.optional.iter().map(|r| ("optional", r)))
                    .chain(p.detection.forbidden.iter().map(|r| ("forbidden", r)))
                    .map(|(group, r)| RuleDetail {
                        group: group.to_string(),
                        rule_type: r.rule_type.clone(),
                        path: r.path.clone(),
                        pattern: r.pattern.clone(),
                        ext: r.ext.clone(),
                        weight: r.weight,
                    })
                    .collect(),
            },
            launch: LaunchDetail {
                strategy: p.launch.strategy.clone(),
                entry_patterns: p.launch.entry_patterns.clone(),
                exclude_patterns: p.launch.exclude_patterns.clone(),
                args: p.launch.args.clone(),
                sandbox_home: p.launch.sandbox_home,
                runtime_id: p.launch.runtime_id.clone(),
                program: p.launch.program.clone(),
            },
            errors: entry.errors.clone(),
        })
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
    #[allow(dead_code)]
    pub fn has_enabled_engine(&self) -> bool {
        self.entries.values().any(|e| e.enabled && e.valid)
    }

    #[allow(dead_code)]
    pub fn engine_count(&self) -> usize {
        self.entries.len()
    }

    #[allow(dead_code)]
    pub fn enabled_count(&self) -> usize {
        self.entries
            .values()
            .filter(|e| e.enabled && e.valid)
            .count()
    }

    /// 返回所有引擎 ID 列表（用于持久化恢复）
    pub fn engine_ids(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
