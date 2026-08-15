use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct ImportDialogState {
    pub open: bool,
    pub entry_path: Option<PathBuf>,
    pub error: Option<String>,
}

impl ImportDialogState {
    pub fn set_entry_path(&mut self, path: PathBuf) {
        self.entry_path = Some(path);
        self.error = None;
    }

    pub fn can_submit(&self) -> bool {
        self.entry_path.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct ScanDialogState {
    pub open: bool,
    pub root: PathBuf,
    pub max_depth: u32,
    pub label: String,
    pub progress: Option<u8>,
}

impl ScanDialogState {
    pub fn open(root: PathBuf, max_depth: u32) -> Self {
        Self {
            open: true,
            root,
            max_depth,
            label: String::new(),
            progress: None,
        }
    }

    pub fn apply_progress(&mut self, label: impl Into<String>, progress: Option<u8>) {
        self.label = label.into();
        self.progress = progress.map(|value| value.min(100));
    }
}

#[derive(Clone, Debug, Default)]
pub struct DialogState {
    pub import: ImportDialogState,
    pub scan: Option<ScanDialogState>,
}
