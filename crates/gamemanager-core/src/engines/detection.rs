use super::{context::DetectionContext, profile::DetectionRuleDefinition};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectionMatch {
    pub engine_id: String,
    pub confidence: i32,
}

pub(crate) fn evaluate_rule(
    rule: &DetectionRuleDefinition,
    context: &dyn DetectionContext,
) -> bool {
    match rule.rule_type.as_str() {
        "file_exists" => context.file_exists(&rule.path),
        "dir_exists" => context.dir_exists(&rule.path),
        "glob_match" => context.glob_match(&rule.pattern),
        "glob_match_recursive" => context.glob_match_recursive(&rule.pattern, 3),
        "has_extension" => context.has_extension(&rule.extension),
        "has_native_executable" => context.has_native_executable(),
        _ => false,
    }
}

pub(crate) fn optional_score(
    rules: &[DetectionRuleDefinition],
    context: &dyn DetectionContext,
) -> i32 {
    rules
        .iter()
        .filter(|rule| evaluate_rule(rule, context))
        .map(|rule| rule.weight)
        .sum()
}

pub(crate) fn confidence_score(rules: &[DetectionRuleDefinition], score: i32) -> i32 {
    let maximum = rules.iter().map(|rule| rule.weight.max(0)).sum::<i32>();
    if maximum == 0 {
        return 0;
    }

    ((score as f64 / maximum as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as i32
}
