//! 引擎配置数据结构 — TOML 反序列化 + 运行时类型。
//!
//! 每个引擎由一个 `engines/<id>.toml` 文件定义，包含检测规则、启动策略和元数据。
//! 此模块负责从 TOML 反序列化为强类型 Rust 结构体。

use serde::Deserialize;

// ─── 顶层结构 ────────────────────────────────────────────

/// 一个完整的引擎描述文件（对应一个 TOML）
#[derive(Debug, Clone, Deserialize)]
pub struct EngineProfile {
    pub meta: EngineMeta,
    #[serde(default)]
    pub detection: DetectionConfig,
    pub launch: LaunchConfig,
}

// ─── 元数据 ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct EngineMeta {
    /// 引擎唯一标识（如 "rpgmakermv"）
    pub id: String,
    /// 显示名称（如 "RPG Maker MV"）
    pub name: String,
    /// 分组类别（如 "nwjs", "renpy", "unity"）
    /// 前端用 category 来合并显示选项
    #[serde(default = "default_category")]
    pub category: String,
    /// Iconify 图标 ID
    #[serde(default = "default_icon")]
    pub icon: String,
    /// 同分时的优先级 breaker（越小越优先）
    #[serde(default)]
    pub priority: i32,
    /// 引擎描述（可选，用于 tooltip）
    #[serde(default)]
    pub description: String,
    /// 扫描时跳过（仅支持手动导入）
    #[serde(default)]
    pub skip_scan: bool,
}

fn default_category() -> String {
    "other".into()
}

fn default_icon() -> String {
    "ri:question-line".into()
}

// ─── 检测配置 ────────────────────────────────────────────

/// 检测配置：规则列表 + 最低得分阈值
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DetectionConfig {
    /// 最低得分（低于此值不匹配）
    #[serde(default)]
    pub min_score: i32,
    /// 检测规则列表
    #[serde(default)]
    pub rules: Vec<DetectionRuleDef>,
}

/// 单个检测规则定义（TOML 中 [[detection.rules]] 条目）
#[derive(Debug, Clone, Deserialize)]
pub struct DetectionRuleDef {
    /// 规则类型
    #[serde(rename = "type")]
    pub rule_type: String,
    /// 相对路径（用于 file_exists / dir_exists）
    #[serde(default)]
    pub path: String,
    /// glob 模式（用于 glob_match）
    #[serde(default)]
    pub pattern: String,
    /// 文件扩展名（用于 has_extension）
    #[serde(default)]
    pub ext: String,
    /// 权重分值
    #[serde(default = "default_weight")]
    pub weight: i32,
}

fn default_weight() -> i32 {
    1
}

impl DetectionRuleDef {
    /// 验证规则定义是否有效
    pub fn validate(&self) -> Result<(), String> {
        match self.rule_type.as_str() {
            "file_exists" | "dir_exists" => {
                if self.path.is_empty() {
                    return Err(format!("{} 规则缺少 path 字段", self.rule_type));
                }
            }
            "glob_match" => {
                if self.pattern.is_empty() {
                    return Err("glob_match 规则缺少 pattern 字段".into());
                }
            }
            "has_extension" => {
                if self.ext.is_empty() {
                    return Err("has_extension 规则缺少 ext 字段".into());
                }
            }
            other => return Err(format!("未知的检测规则类型: {}", other)),
        }
        if self.weight < 0 {
            return Err("规则 weight 不能为负数".into());
        }
        Ok(())
    }
}

// ─── 启动配置 ────────────────────────────────────────────

/// 启动配置：策略类型 + 策略参数
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LaunchConfig {
    /// 启动策略类型：native | nwjs | external
    pub strategy: String,

    /// 入口可执行文件匹配模式（按优先级排列）
    #[serde(default)]
    pub entry_patterns: Vec<String>,

    /// 排除的可执行文件名 glob 模式（如 "UnityCrashHandler*.exe"）
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// 启动参数
    #[serde(default)]
    pub args: Vec<String>,

    /// 是否启用 HOME 沙箱隔离
    #[serde(default)]
    pub sandbox_home: bool,

    // ── NW.js 专用字段 ──
    /// NW.js 运行时 ID（仅在 strategy = "nwjs" 时需要）
    #[serde(default)]
    pub runtime_id: String,

    // ── External 专用字段 ──
    /// 外部程序名（仅在 strategy = "external" 时需要）
    #[serde(default)]
    pub program: String,
    /// 外部程序参数前缀（如 flatpak run --command=bottles-cli ...）
    #[serde(default)]
    pub program_args_prefix: Vec<String>,
    /// 关联的集成服务 key（如 "bottles"）
    #[serde(default)]
    pub required_integration: String,
    /// 参数模板：{exe}, {bottle} 等占位符
    #[serde(default)]
    pub args_template: String,

    // ── 沙箱配置 ──
    /// 需要保留的目录（相对于游戏目录）
    #[serde(default)]
    pub preserve_dirs: Vec<String>,

    // ── 附加配置（引擎特定键值对，通过 metadata_json 透传）──
    /// 引擎特定的额外键值对
    #[serde(default)]
    pub extras: std::collections::HashMap<String, String>,
}

impl LaunchConfig {
    /// 验证启动配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        match self.strategy.as_str() {
            "native" => {}
            "nwjs" => {
                if self.runtime_id.is_empty() {
                    return Err("nwjs 策略缺少 runtime_id 字段".into());
                }
            }
            "external" => {
                if self.program.is_empty() {
                    return Err("external 策略缺少 program 字段".into());
                }
            }
            other => return Err(format!("未知的启动策略: {}", other)),
        }
        Ok(())
    }
}

// ─── 前端用 DTO ──────────────────────────────────────────

/// 传给前端的引擎摘要信息
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineMetaDto {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon: String,
    pub priority: i32,
    pub description: String,
    #[serde(default)]
    pub enabled: bool,
    pub entry_patterns: Vec<String>,
}

impl From<&EngineProfile> for EngineMetaDto {
    fn from(p: &EngineProfile) -> Self {
        Self {
            id: p.meta.id.clone(),
            name: p.meta.name.clone(),
            category: p.meta.category.clone(),
            icon: p.meta.icon.clone(),
            priority: p.meta.priority,
            description: p.meta.description.clone(),
            enabled: true,
            entry_patterns: p.launch.entry_patterns.clone(),
        }
    }
}

/// 插件管理面板用的完整信息
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineDetailDto {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon: String,
    pub description: String,
    pub enabled: bool,
    pub valid: bool,
    pub rule_count: usize,
    pub strategy: String,
    pub errors: Vec<String>,
}

/// 插件详情（查看完整配置用）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineProfileDetailDto {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon: String,
    pub description: String,
    pub enabled: bool,
    pub valid: bool,
    pub detection: DetectionDetail,
    pub launch: LaunchDetail,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionDetail {
    pub min_score: i32,
    pub rules: Vec<RuleDetail>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDetail {
    pub rule_type: String,
    pub path: String,
    pub pattern: String,
    pub ext: String,
    pub weight: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchDetail {
    pub strategy: String,
    pub entry_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub args: Vec<String>,
    pub sandbox_home: bool,
    pub runtime_id: String,
    pub program: String,
}

// ─── 测试 ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mv_profile() {
        let toml_str = r#"
[meta]
id = "rpgmakermv"
name = "RPG Maker MV"
category = "nwjs"
icon = "ri:window-line"
priority = 2

[detection]
min_score = 4

[[detection.rules]]
type = "file_exists"
path = "js/rpg_core.js"
weight = 3

[[detection.rules]]
type = "file_exists"
path = "data/System.json"
weight = 2

[launch]
strategy = "nwjs"
runtime_id = "nwjs-sdk"
entry_patterns = ["nw.exe", "Game.exe"]
args = ["--enable-webgl"]
sandbox_home = true
"#;

        let profile: EngineProfile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.meta.id, "rpgmakermv");
        assert_eq!(profile.meta.category, "nwjs");
        assert_eq!(profile.detection.rules.len(), 2);
        assert_eq!(profile.detection.min_score, 4);
        assert_eq!(profile.launch.strategy, "nwjs");
        assert_eq!(profile.launch.runtime_id, "nwjs-sdk");
    }

    #[test]
    fn parse_renpy_profile() {
        let toml_str = r#"
[meta]
id = "renpy"
name = "Ren'Py"
category = "renpy"
icon = "ri:slideshow-line"

[detection]
min_score = 4

[[detection.rules]]
type = "dir_exists"
path = "renpy"
weight = 3

[[detection.rules]]
type = "has_extension"
ext = "rpy"
weight = 3

[launch]
strategy = "native"
entry_patterns = ["renpy.sh", "renpy.exe"]
"#;

        let profile: EngineProfile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.meta.id, "renpy");
        assert_eq!(profile.launch.strategy, "native");
        assert_eq!(profile.detection.rules.len(), 2);
    }

    #[test]
    fn parse_external_profile() {
        let toml_str = r#"
[meta]
id = "unity-wine"
name = "Unity (Wine)"
category = "other"
icon = "ri:gamepad-line"

[detection]
min_score = 4

[[detection.rules]]
type = "glob_match"
pattern = "*_Data"
weight = 2

[launch]
strategy = "external"
program = "bottles-cli"
program_args_prefix = ["run", "-b"]
args_template = "{bottle} -e {exe}"
required_integration = "bottles"
sandbox_home = true
"#;

        let profile: EngineProfile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.launch.strategy, "external");
        assert_eq!(profile.launch.program, "bottles-cli");
        assert_eq!(profile.launch.required_integration, "bottles");
    }

    #[test]
    fn validate_valid_rules() {
        let def = DetectionRuleDef {
            rule_type: "file_exists".into(),
            path: "test.txt".into(),
            pattern: String::new(),
            ext: String::new(),
            weight: 1,
        };
        assert!(def.validate().is_ok());
    }

    #[test]
    fn validate_missing_field() {
        let def = DetectionRuleDef {
            rule_type: "file_exists".into(),
            path: String::new(),
            pattern: String::new(),
            ext: String::new(),
            weight: 1,
        };
        assert!(def.validate().is_err());
    }

    #[test]
    fn validate_unknown_rule_type() {
        let def = DetectionRuleDef {
            rule_type: "run_arbitrary_code".into(),
            path: String::new(),
            pattern: String::new(),
            ext: String::new(),
            weight: 1,
        };
        assert!(def.validate().is_err());
    }

    #[test]
    fn validate_launch_config_native() {
        let config = LaunchConfig {
            strategy: "native".into(),
            entry_patterns: vec!["game.exe".into()],
            exclude_patterns: vec![],
            args: vec![],
            sandbox_home: false,
            runtime_id: String::new(),
            program: String::new(),
            program_args_prefix: vec![],
            required_integration: String::new(),
            args_template: String::new(),
            preserve_dirs: vec![],
            extras: std::collections::HashMap::new(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_launch_config_nwjs_missing_runtime() {
        let config = LaunchConfig {
            strategy: "nwjs".into(),
            entry_patterns: vec![],
            exclude_patterns: vec![],
            args: vec![],
            sandbox_home: false,
            runtime_id: String::new(), // missing!
            program: String::new(),
            program_args_prefix: vec![],
            required_integration: String::new(),
            args_template: String::new(),
            preserve_dirs: vec![],
            extras: std::collections::HashMap::new(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn engine_meta_dto_conversion() {
        let profile = EngineProfile {
            meta: EngineMeta {
                id: "test".into(),
                name: "Test Engine".into(),
                category: "test".into(),
                icon: "ri:test-line".into(),
                priority: 0,
                description: "A test engine".into(),
                skip_scan: false,
            },
            detection: DetectionConfig {
                min_score: 0,
                rules: vec![],
            },
            launch: LaunchConfig {
                strategy: "native".into(),
                entry_patterns: vec![],
                exclude_patterns: vec![],
                args: vec![],
                sandbox_home: false,
                runtime_id: String::new(),
                program: String::new(),
                program_args_prefix: vec![],
                required_integration: String::new(),
                args_template: String::new(),
                preserve_dirs: vec![],
                extras: std::collections::HashMap::new(),
            },
        };

        let dto = EngineMetaDto::from(&profile);
        assert_eq!(dto.id, "test");
        assert_eq!(dto.name, "Test Engine");
    }
}
