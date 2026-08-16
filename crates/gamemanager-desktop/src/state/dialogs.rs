use std::path::PathBuf;

use gamemanager_core::{AppSettings, OperationId};

use super::GameSettingsState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UtilityDialog {
    Runtime,
    Engines,
    Appearance,
}

#[derive(Clone, Debug, Default)]
pub struct AppearanceDialogState {
    pub container_root: String,
    pub saving_root: bool,
    pub cleaning_profiles: bool,
    pub removing_games: bool,
    pub confirm_remove_all: bool,
    pub error: Option<String>,
}

impl AppearanceDialogState {
    pub fn from_settings(settings: AppSettings) -> Self {
        Self {
            container_root: settings.container_root,
            ..Self::default()
        }
    }

    pub fn set_container_root(&mut self, path: impl Into<String>) {
        self.container_root = path.into();
        self.error = None;
    }

    pub fn is_busy(&self) -> bool {
        self.saving_root || self.cleaning_profiles || self.removing_games
    }
}

#[derive(Clone, Debug, Default)]
pub struct ImportDialogState {
    pub open: bool,
    pub drop_active: bool,
    pub entry_path: Option<PathBuf>,
    pub error: Option<String>,
    pub submitting: bool,
}

impl ImportDialogState {
    pub fn begin_drop(&mut self) {
        self.drop_active = true;
    }

    pub fn end_drop(&mut self) {
        self.drop_active = false;
    }

    pub fn accept_dropped_entry(&mut self, path: PathBuf) {
        self.open = true;
        self.drop_active = false;
        self.set_entry_path(path);
    }

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

    pub fn set_max_depth_text(&mut self, value: &str) {
        if let Ok(value) = value.trim().parse::<u32>() {
            self.max_depth = value.clamp(1, 10);
        }
    }

    pub fn adjust_max_depth(&mut self, delta: i8) {
        self.max_depth = self
            .max_depth
            .saturating_add_signed(i32::from(delta))
            .clamp(1, 10);
    }

    pub fn can_submit(&self) -> bool {
        self.root.is_dir() && self.operation_id.is_none()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DialogState {
    pub app_menu_open: bool,
    pub import: ImportDialogState,
    pub scan: Option<ScanDialogState>,
    pub settings: Option<GameSettingsState>,
    pub delete: Option<DeleteDialogState>,
    pub appearance: AppearanceDialogState,
    pub utility: Option<UtilityDialog>,
}

impl DialogState {
    pub fn open_utility(&mut self, utility: UtilityDialog) {
        self.app_menu_open = false;
        self.utility = Some(utility);
    }

    pub fn close_utility(&mut self) {
        if self.utility == Some(UtilityDialog::Appearance) && self.appearance.is_busy() {
            return;
        }
        self.utility = None;
    }

    pub fn dismiss_non_busy(&mut self) -> bool {
        if self.app_menu_open {
            self.app_menu_open = false;
            return true;
        }
        if self.utility.is_some() {
            let previous = self.utility;
            self.close_utility();
            if self.utility != previous {
                return true;
            }
        }
        if self.delete.as_ref().is_some_and(|delete| !delete.deleting) {
            self.delete = None;
            return true;
        }
        if self
            .settings
            .as_ref()
            .is_some_and(|settings| !settings.saving)
        {
            self.settings = None;
            return true;
        }
        if self
            .scan
            .as_ref()
            .is_some_and(|scan| scan.operation_id.is_none())
        {
            self.scan = None;
            return true;
        }
        if self.import.open && !self.import.submitting {
            self.import.open = false;
            return true;
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct DeleteDialogState {
    pub game_id: String,
    pub title: String,
    pub error: Option<String>,
    pub deleting: bool,
}

impl DeleteDialogState {
    pub fn new(game_id: String, title: String) -> Self {
        Self {
            game_id,
            title,
            error: None,
            deleting: false,
        }
    }
}
