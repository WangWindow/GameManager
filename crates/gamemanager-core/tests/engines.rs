use std::{collections::BTreeMap, fs, path::Path};

use gamemanager_core::{DetectionContext, EngineRegistry, EngineRuleRequirement};
use tempfile::TempDir;

#[test]
fn invalid_toml_is_reported_without_hiding_valid_engines() {
    let engines = TestEngineDirectory::new(&[
        ("html.toml", HTML_PROFILE),
        ("broken.toml", "[meta]\nid = \"broken\"\nunknown = true\n"),
    ]);

    let report = EngineRegistry::load(engines.path(), &BTreeMap::new());

    assert!(report.warnings.iter().any(|warning| warning.id == "broken"));
    assert!(report.registry.summary("html").is_some());
}

#[test]
fn specific_match_beats_other_at_the_same_score() {
    let engines =
        TestEngineDirectory::new(&[("html.toml", HTML_PROFILE), ("other.toml", OTHER_PROFILE)]);
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;

    assert_eq!(
        registry
            .detect(&HtmlContext)
            .expect("HTML should match")
            .engine_id,
        "html"
    );
}

#[test]
fn disabling_an_engine_removes_it_from_detection() {
    let engines = TestEngineDirectory::new(&[("html.toml", HTML_PROFILE)]);
    let mut registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;

    registry.set_enabled("html", false).expect("known engine");
    assert!(registry.detect(&HtmlContext).is_none());
}

#[test]
fn all_built_in_profiles_synchronize_and_validate() {
    let directory = tempfile::tempdir().expect("temporary engine directory");
    EngineRegistry::synchronize_builtin_profiles(directory.path()).expect("copy built-ins");

    for name in EngineRegistry::builtin_profile_names() {
        assert!(directory.path().join(name).is_file(), "missing {name}");
    }

    let report = EngineRegistry::load(directory.path(), &BTreeMap::new());
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(
        report.registry.summaries().len(),
        EngineRegistry::builtin_profile_names().count()
    );
}

#[test]
fn details_expose_detection_and_launch_information_for_the_engine_dialog() {
    let directory = tempfile::tempdir().expect("temporary engine directory");
    EngineRegistry::synchronize_builtin_profiles(directory.path()).expect("copy built-ins");
    let registry = EngineRegistry::load(directory.path(), &BTreeMap::new()).registry;
    let html = registry
        .details()
        .into_iter()
        .find(|detail| detail.summary.id == "html")
        .expect("HTML profile");

    assert_eq!(html.minimum_score, 0);
    assert!(
        html.rules
            .iter()
            .any(|rule| matches!(rule.requirement, EngineRuleRequirement::Required))
    );
    assert!(!html.summary.entry_patterns.is_empty());
}

const HTML_PROFILE: &str = r#"
[meta]
id = "html"
name = "HTML"
category = "nwjs"
priority = 4

[detection]
min_score = 0

[[detection.required]]
type = "file_exists"
path = "index.html"

[launch]
strategy = "nwjs"
runtime_id = "nwjs-sdk"
"#;

const OTHER_PROFILE: &str = r#"
[meta]
id = "other"
name = "Other"
category = "other"
priority = 0

[detection]
min_score = 0

[[detection.required]]
type = "file_exists"
path = "index.html"

[launch]
strategy = "bottles"
"#;

struct TestEngineDirectory(TempDir);

impl TestEngineDirectory {
    fn new(files: &[(&str, &str)]) -> Self {
        let directory = tempfile::tempdir().expect("temporary engine directory");
        for (name, contents) in files {
            fs::write(directory.path().join(name), contents).expect("engine profile");
        }
        Self(directory)
    }

    fn path(&self) -> &Path {
        self.0.path()
    }
}

struct HtmlContext;

impl DetectionContext for HtmlContext {
    fn file_exists(&self, relative_path: &str) -> bool {
        relative_path == "index.html"
    }

    fn dir_exists(&self, _: &str) -> bool {
        false
    }

    fn glob_match(&self, _: &str) -> bool {
        false
    }

    fn has_extension(&self, _: &str) -> bool {
        false
    }

    fn has_native_executable(&self) -> bool {
        false
    }

    fn game_dir(&self) -> &Path {
        Path::new(".")
    }
}
