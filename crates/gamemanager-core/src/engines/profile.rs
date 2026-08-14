use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EngineProfile {
    pub meta: EngineMeta,
    #[serde(default)]
    pub detection: DetectionConfig,
    pub launch: LaunchConfig,
}

impl EngineProfile {
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = self.detection.validation_errors();
        errors.extend(self.launch.validation_errors());
        if self.meta.id.trim().is_empty() {
            errors.push("engine id must not be empty".to_owned());
        }
        errors
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EngineMeta {
    pub id: String,
    pub name: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub skip_scan: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DetectionConfig {
    #[serde(default)]
    pub min_score: i32,
    #[serde(default)]
    pub required: Vec<DetectionRuleDefinition>,
    #[serde(default)]
    pub optional: Vec<DetectionRuleDefinition>,
    #[serde(default)]
    pub forbidden: Vec<DetectionRuleDefinition>,
}

impl DetectionConfig {
    pub fn rule_count(&self) -> usize {
        self.required.len() + self.optional.len() + self.forbidden.len()
    }

    fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.rule_count() == 0 {
            errors.push("detection requires at least one rule".to_owned());
        }
        if self.min_score < 0 {
            errors.push("detection min_score must not be negative".to_owned());
        }

        let maximum = self
            .optional
            .iter()
            .map(|rule| rule.weight.max(0))
            .sum::<i32>();
        if self.min_score > maximum {
            errors.push(format!(
                "detection min_score ({}) exceeds optional rule total ({maximum})",
                self.min_score
            ));
        }

        for rule in self
            .required
            .iter()
            .chain(&self.optional)
            .chain(&self.forbidden)
        {
            errors.extend(rule.validation_errors());
        }
        errors
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DetectionRuleDefinition {
    #[serde(rename = "type")]
    pub rule_type: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default, rename = "ext")]
    pub extension: String,
    #[serde(default = "default_weight")]
    pub weight: i32,
}

impl DetectionRuleDefinition {
    fn validation_errors(&self) -> Vec<String> {
        let missing = match self.rule_type.as_str() {
            "file_exists" | "dir_exists" if self.path.is_empty() => Some("path"),
            "glob_match" | "glob_match_recursive" if self.pattern.is_empty() => Some("pattern"),
            "has_extension" if self.extension.is_empty() => Some("ext"),
            "file_exists"
            | "dir_exists"
            | "glob_match"
            | "glob_match_recursive"
            | "has_extension"
            | "has_native_executable" => None,
            _ => return vec![format!("unknown detection rule type: {}", self.rule_type)],
        };

        let mut errors = Vec::new();
        if let Some(field) = missing {
            errors.push(format!("{} rule requires {field}", self.rule_type));
        }
        if self.weight < 0 {
            errors.push(format!(
                "{} rule weight must not be negative",
                self.rule_type
            ));
        }
        errors
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchConfig {
    pub strategy: String,
    #[serde(default)]
    pub entry_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub sandbox_home: bool,
    #[serde(default)]
    pub runtime_id: String,
    #[serde(default)]
    pub program: String,
    #[serde(default)]
    pub program_args_prefix: Vec<String>,
    #[serde(default)]
    pub required_integration: String,
    #[serde(default)]
    pub args_template: String,
    #[serde(default)]
    pub preserve_dirs: Vec<String>,
    #[serde(default)]
    pub extras: BTreeMap<String, String>,
}

impl LaunchConfig {
    fn validation_errors(&self) -> Vec<String> {
        match self.strategy.as_str() {
            "native" | "bottles" | "mkxpz" => Vec::new(),
            "nwjs" if self.runtime_id.is_empty() => {
                vec!["nwjs launch strategy requires runtime_id".to_owned()]
            }
            "nwjs" => Vec::new(),
            "external" if self.program.is_empty() => {
                vec!["external launch strategy requires program".to_owned()]
            }
            "external" => Vec::new(),
            _ => vec![format!("unknown launch strategy: {}", self.strategy)],
        }
    }
}

fn default_category() -> String {
    "other".to_owned()
}

fn default_icon() -> String {
    "ri:question-line".to_owned()
}

fn default_weight() -> i32 {
    1
}
