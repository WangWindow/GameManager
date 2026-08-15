use std::path::PathBuf;

use gamemanager_core::OperationId;

use super::GameSettingsState;

#[derive(Clone, Debug, Default)]
pub struct ImportDialogState {
    pub open: bool,
    pub entry_path: Option<PathBuf>,
    pub error: Option<String>,
    pub submitting: bool,
}

impl ImportDialogState {
    pub fn set_entry_path(&mut self, path: PathBuf) {
        self.entry_path = Some(path);
        self.error = None;
    }

    pub fn can_submit(&self) -> bool {
        self.entry_path.is_some() && !self.submitting
    }
}

#[derive(Clone, Debug)]
pub struct ScanDialogState {
    pub open: bool,
    pub root: PathBuf,
    pub max_depth: u32,
    pub label: String,
    pub progress: Option<u8>,
    pub operation_id: Option<OperationId>,
    pub error: Option<String>,
}

impl ScanDialogState {
    pub fn open(root: PathBuf, max_depth: u32) -> Self {
        Self {
            open: true,
            root,
            max_depth,
            label: String::new(),
            progress: None,
            operation_id: None,
            error: None,
        }
    }

    pub fn apply_progress(&mut self, label: impl Into<String>, progress: Option<u8>) {
        self.label = label.into();
        self.progress = progress.map(|value| value.min(100));
    }

    pub fn set_operation(&mut self, id: OperationId) {
        self.operation_id = Some(id);
        self.error = None;
    }
}

#[derive(Clone, Debug, Default)]
pub struct DialogState {
    pub import: ImportDialogState,
    pub scan: Option<ScanDialogState>,
    pub settings: Option<GameSettingsState>,
}
