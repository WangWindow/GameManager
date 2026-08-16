use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{CoreError, Result};

use super::{
    context::{DetectionContext, is_native_executable, simple_glob_match},
    detection::{DetectionMatch, confidence_score, evaluate_rule, optional_score},
    profile::EngineProfile,
};

const BUILTIN_PROFILES: &[(&str, &str)] = &[
    (
        "electron.toml",
        include_str!("../../assets/engines/electron.toml"),
    ),
    (
        "godot.toml",
        include_str!("../../assets/engines/godot.toml"),
    ),
    ("html.toml", include_str!("../../assets/engines/html.toml")),
    (
        "other.toml",
        include_str!("../../assets/engines/other.toml"),
    ),
    (
        "renpy.toml",
        include_str!("../../assets/engines/renpy.toml"),
    ),
    (
        "rpgmakermv.toml",
        include_str!("../../assets/engines/rpgmakermv.toml"),
    ),
    (
        "rpgmakervx.toml",
        include_str!("../../assets/engines/rpgmakervx.toml"),
    ),
    (
        "unity.toml",
        include_str!("../../assets/engines/unity.toml"),
    ),
    (
        "unreal.toml",
        include_str!("../../assets/engines/unreal.toml"),
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryWarning {
    pub id: String,
    pub message: String,
}

pub struct RegistryReport {
    pub registry: EngineRegistry,
    pub warnings: Vec<RegistryWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineSummary {
    pub id: String,
    pub name: String,
    pub category: String,
    pub priority: i32,
    pub description: String,
    pub enabled: bool,
    pub entry_patterns: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineDetail {
    pub summary: EngineSummary,
    pub valid: bool,
    pub rule_count: usize,
    pub minimum_score: i32,
    pub rules: Vec<EngineRuleSummary>,
    pub strategy: String,
    pub exclude_patterns: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineRuleRequirement {
    Required,
    Optional,
    Forbidden,
}

impl EngineRuleRequirement {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
            Self::Forbidden => "forbidden",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineRuleSummary {
    pub requirement: EngineRuleRequirement,
    pub rule_type: String,
    pub target: String,
    pub weight: Option<i32>,
}

#[derive(Clone, Debug)]
struct EngineEntry {
    profile: EngineProfile,
    enabled: bool,
    valid: bool,
    errors: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct EngineRegistry {
    entries: BTreeMap<String, EngineEntry>,
}

impl EngineRegistry {
    /// Copies the embedded built-in profiles into the v0.9 data location.
    ///
    /// User-created profiles are left intact, while profiles shipped by the
    /// application are refreshed on each bootstrap exactly as in v0.9.
    pub fn synchronize_builtin_profiles(directory: &Path) -> Result<()> {
        fs::create_dir_all(directory)?;
        for (file_name, contents) in BUILTIN_PROFILES {
            fs::write(directory.join(file_name), contents)?;
        }
        Ok(())
    }

    pub fn builtin_profile_names() -> impl Iterator<Item = &'static str> {
        BUILTIN_PROFILES.iter().map(|(name, _)| *name)
    }

    /// Loads every TOML profile, retaining valid profiles if neighboring files
    /// have validation or parsing errors.
    pub fn load(directory: &Path, enabled: &BTreeMap<String, bool>) -> RegistryReport {
        let mut registry = Self::default();
        let mut warnings = Vec::new();
        let mut paths = match fs::read_dir(directory) {
            Ok(entries) => entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "toml")
                })
                .collect::<Vec<_>>(),
            Err(error) => {
                warnings.push(RegistryWarning {
                    id: "registry".to_owned(),
                    message: format!("could not read {}: {error}", directory.display()),
                });
                return RegistryReport { registry, warnings };
            }
        };
        paths.sort();

        for path in paths {
            registry.load_one(&path, enabled, &mut warnings);
        }

        RegistryReport { registry, warnings }
    }

    fn load_one(
        &mut self,
        path: &Path,
        enabled: &BTreeMap<String, bool>,
        warnings: &mut Vec<RegistryWarning>,
    ) {
        let fallback_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown")
            .to_owned();
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) => {
                warnings.push(RegistryWarning {
                    id: fallback_id,
                    message: format!("could not read {}: {error}", path.display()),
                });
                return;
            }
        };
        let profile = match toml::from_str::<EngineProfile>(&content) {
            Ok(profile) => profile,
            Err(error) => {
                warnings.push(RegistryWarning {
                    id: fallback_id,
                    message: format!("could not parse {}: {error}", path.display()),
                });
                return;
            }
        };

        let id = profile.meta.id.clone();
        let mut errors = profile.validation_errors();
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(id.as_str()) {
            errors.push(format!("profile id '{id}' must match its file name"));
        }
        if self.entries.contains_key(&id) {
            errors.push(format!("duplicate engine profile id: {id}"));
        }

        let valid = errors.is_empty();
        if !valid {
            warnings.push(RegistryWarning {
                id: id.clone(),
                message: errors.join("; "),
            });
        }
        if self.entries.contains_key(&id) {
            return;
        }

        let enabled = enabled.get(&id).copied().unwrap_or(true) && valid;
        self.entries.insert(
            id,
            EngineEntry {
                profile,
                enabled,
                valid,
                errors,
            },
        );
    }

    pub fn detect(&self, context: &dyn DetectionContext) -> Option<DetectionMatch> {
        self.detect_matching(context, |_| true)
    }

    /// Detects the best engine that is eligible for automatic directory scans.
    /// Engines marked `skip_scan` remain available for manual imports, but
    /// must not mask another engine that can scan the same directory.
    pub fn detect_for_scan(&self, context: &dyn DetectionContext) -> Option<DetectionMatch> {
        self.detect_matching(context, |entry| !entry.profile.meta.skip_scan)
    }

    fn detect_matching(
        &self,
        context: &dyn DetectionContext,
        include: impl Fn(&EngineEntry) -> bool,
    ) -> Option<DetectionMatch> {
        let mut best_specific: Option<Candidate> = None;
        let mut best_other: Option<Candidate> = None;

        for entry in self.entries.values() {
            if !entry.enabled || !entry.valid || !include(entry) {
                continue;
            }
            if entry
                .profile
                .detection
                .required
                .iter()
                .any(|rule| !evaluate_rule(rule, context))
                || entry
                    .profile
                    .detection
                    .forbidden
                    .iter()
                    .any(|rule| evaluate_rule(rule, context))
            {
                continue;
            }

            let raw_score = optional_score(&entry.profile.detection.optional, context);
            if raw_score < entry.profile.detection.min_score {
                continue;
            }

            let confidence = if entry.profile.detection.required.is_empty() {
                confidence_score(&entry.profile.detection.optional, raw_score).max(1)
            } else if entry.profile.detection.optional.is_empty() {
                100
            } else {
                60 + confidence_score(&entry.profile.detection.optional, raw_score) * 40 / 100
            };
            let candidate = Candidate {
                engine_id: entry.profile.meta.id.clone(),
                confidence,
                raw_score,
                priority: entry.profile.meta.priority,
            };
            let target = if candidate.engine_id == "other" {
                &mut best_other
            } else {
                &mut best_specific
            };
            if target
                .as_ref()
                .is_none_or(|current| candidate.is_better_than(current))
            {
                *target = Some(candidate);
            }
        }

        best_specific
            .or(best_other)
            .map(|candidate| DetectionMatch {
                engine_id: candidate.engine_id,
                confidence: candidate.confidence,
            })
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<()> {
        let entry = self
            .entries
            .get_mut(id)
            .ok_or_else(|| CoreError::Engine(format!("engine not found: {id}")))?;
        if enabled && !entry.valid {
            return Err(CoreError::Engine(format!(
                "engine cannot be enabled: {}",
                entry.errors.join("; ")
            )));
        }
        entry.enabled = enabled;
        Ok(())
    }

    pub fn summary(&self, id: &str) -> Option<EngineSummary> {
        self.entries.get(id).map(EngineEntry::summary)
    }

    pub fn summaries(&self) -> Vec<EngineSummary> {
        let mut summaries = self
            .entries
            .values()
            .map(EngineEntry::summary)
            .collect::<Vec<_>>();
        summaries.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
        summaries
    }

    pub fn details(&self) -> Vec<EngineDetail> {
        let mut details = self
            .entries
            .values()
            .map(|entry| EngineDetail {
                summary: entry.summary(),
                valid: entry.valid,
                rule_count: entry.profile.detection.rule_count(),
                minimum_score: entry.profile.detection.min_score,
                rules: entry.rule_summaries(),
                strategy: entry.profile.launch.strategy.clone(),
                exclude_patterns: entry.profile.launch.exclude_patterns.clone(),
                errors: entry.errors.clone(),
            })
            .collect::<Vec<_>>();
        details.sort_by(|left, right| left.summary.id.cmp(&right.summary.id));
        details
    }

    pub fn profile(&self, id: &str) -> Option<&EngineProfile> {
        self.entries.get(id).map(|entry| &entry.profile)
    }

    /// Resolves the launch entry for an already-detected game root. The
    /// profile order is significant: a native entry wins over a Windows
    /// executable when both are present, matching manual import behaviour.
    pub fn resolve_entry(&self, id: &str, game_root: &Path) -> Option<PathBuf> {
        let profile = self.profile(id)?;
        if profile.launch.entry_patterns.is_empty() {
            return game_root.is_dir().then(|| game_root.to_path_buf());
        }

        let mut entries = fs::read_dir(game_root)
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        entries.sort();

        for pattern in &profile.launch.entry_patterns {
            let candidate = if pattern == "@native" {
                entries.iter().find(|path| {
                    is_native_executable(path)
                        && !is_excluded(path, &profile.launch.exclude_patterns)
                })
            } else {
                entries.iter().find(|path| {
                    let name = path.file_name().and_then(|name| name.to_str());
                    name.is_some_and(|name| {
                        simple_glob_match(pattern, name)
                            && !is_excluded(path, &profile.launch.exclude_patterns)
                    })
                })
            };
            if let Some(candidate) = candidate {
                return Some(candidate.clone());
            }
        }

        None
    }

    pub fn should_skip_scan(&self, id: &str) -> bool {
        self.entries
            .get(id)
            .is_some_and(|entry| entry.profile.meta.skip_scan)
    }
}

fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    patterns
        .iter()
        .any(|pattern| simple_glob_match(pattern, name))
}

impl EngineEntry {
    fn summary(&self) -> EngineSummary {
        EngineSummary {
            id: self.profile.meta.id.clone(),
            name: self.profile.meta.name.clone(),
            category: self.profile.meta.category.clone(),
            priority: self.profile.meta.priority,
            description: self.profile.meta.description.clone(),
            enabled: self.enabled,
            entry_patterns: self.profile.launch.entry_patterns.clone(),
        }
    }

    fn rule_summaries(&self) -> Vec<EngineRuleSummary> {
        let detection = &self.profile.detection;
        detection
            .required
            .iter()
            .map(|rule| EngineRuleSummary::from_rule(EngineRuleRequirement::Required, rule))
            .chain(
                detection.optional.iter().map(|rule| {
                    EngineRuleSummary::from_rule(EngineRuleRequirement::Optional, rule)
                }),
            )
            .chain(
                detection.forbidden.iter().map(|rule| {
                    EngineRuleSummary::from_rule(EngineRuleRequirement::Forbidden, rule)
                }),
            )
            .collect()
    }
}

impl EngineRuleSummary {
    fn from_rule(
        requirement: EngineRuleRequirement,
        rule: &super::profile::DetectionRuleDefinition,
    ) -> Self {
        let target = if !rule.path.is_empty() {
            rule.path.clone()
        } else if !rule.pattern.is_empty() {
            rule.pattern.clone()
        } else if !rule.extension.is_empty() {
            rule.extension.clone()
        } else {
            String::new()
        };

        Self {
            requirement,
            rule_type: rule.rule_type.clone(),
            target,
            weight: (requirement == EngineRuleRequirement::Optional).then_some(rule.weight),
        }
    }
}

#[derive(Clone, Debug)]
struct Candidate {
    engine_id: String,
    confidence: i32,
    raw_score: i32,
    priority: i32,
}

impl Candidate {
    fn is_better_than(&self, other: &Self) -> bool {
        self.confidence > other.confidence
            || (self.confidence == other.confidence && self.raw_score > other.raw_score)
            || (self.confidence == other.confidence
                && self.raw_score == other.raw_score
                && self.priority < other.priority)
            || (self.confidence == other.confidence
                && self.raw_score == other.raw_score
                && self.priority == other.priority
                && self.engine_id < other.engine_id)
    }
}
