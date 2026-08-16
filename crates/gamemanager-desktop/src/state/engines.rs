use gamemanager_core::{EngineDetail, EngineRuleSummary, EngineSummary};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineRow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub valid: bool,
    pub rule_count: usize,
    pub minimum_score: i32,
    pub rules: Vec<EngineRuleSummary>,
    pub strategy: String,
    pub entry_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct EngineListState {
    entries: Vec<EngineRow>,
    expanded_engine_id: Option<String>,
}

impl EngineListState {
    pub fn with_entries(entries: Vec<EngineRow>) -> Self {
        Self {
            entries,
            expanded_engine_id: None,
        }
    }

    pub fn from_details(details: Vec<EngineDetail>) -> Self {
        Self::with_entries(details.into_iter().map(EngineRow::from).collect())
    }

    pub fn entries(&self) -> &[EngineRow] {
        &self.entries
    }

    pub fn entry(&self, id: &str) -> Option<&EngineRow> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn display_name(&self, id: &str) -> String {
        self.entry(id).map_or_else(
            || match id {
                "other" => "Other".to_owned(),
                _ => id.to_owned(),
            },
            |entry| entry.name.clone(),
        )
    }

    pub fn apply_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.enabled = enabled;
        }
    }

    pub fn toggle_expanded(&mut self, id: &str) {
        if self.expanded_engine_id.as_deref() == Some(id) {
            self.expanded_engine_id = None;
        } else if self.entry(id).is_some() {
            self.expanded_engine_id = Some(id.to_owned());
        }
    }

    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded_engine_id.as_deref() == Some(id)
    }
}

impl From<EngineDetail> for EngineRow {
    fn from(detail: EngineDetail) -> Self {
        Self {
            id: detail.summary.id,
            name: detail.summary.name,
            enabled: detail.summary.enabled,
            valid: detail.valid,
            rule_count: detail.rule_count,
            minimum_score: detail.minimum_score,
            rules: detail.rules,
            strategy: detail.strategy,
            entry_patterns: detail.summary.entry_patterns,
            exclude_patterns: detail.exclude_patterns,
            errors: detail.errors,
        }
    }
}

impl From<EngineSummary> for EngineRow {
    fn from(summary: EngineSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            enabled: summary.enabled,
            valid: true,
            rule_count: summary.entry_patterns.len(),
            minimum_score: 0,
            rules: Vec::new(),
            strategy: String::new(),
            entry_patterns: summary.entry_patterns,
            exclude_patterns: Vec::new(),
            errors: Vec::new(),
        }
    }
}
