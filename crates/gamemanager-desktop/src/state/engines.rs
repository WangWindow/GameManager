use gamemanager_core::{EngineDetail, EngineSummary};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineRow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub valid: bool,
    pub rule_count: usize,
    pub strategy: String,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct EngineListState {
    entries: Vec<EngineRow>,
}

impl EngineListState {
    pub fn with_entries(entries: Vec<EngineRow>) -> Self {
        Self { entries }
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

    pub fn apply_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.enabled = enabled;
        }
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
            strategy: detail.strategy,
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
            strategy: String::new(),
            errors: Vec::new(),
        }
    }
}
