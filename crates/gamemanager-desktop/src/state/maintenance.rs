use gamemanager_core::{OperationId, RuntimeStatus};

#[derive(Clone, Debug, Default)]
pub struct MaintenanceState {
    runtimes: Vec<RuntimeStatus>,
    bottles: Vec<String>,
    bottles_default: Option<String>,
    bottles_available: bool,
    bottles_enabled: bool,
    bottles_error: Option<String>,
    bottles_loading: bool,
    runtime_loading: bool,
    runtime_operation_id: Option<OperationId>,
    pub error: Option<String>,
}

impl MaintenanceState {
    pub fn with_runtimes(runtimes: Vec<RuntimeStatus>) -> Self {
        Self {
            runtimes,
            bottles: Vec::new(),
            bottles_default: None,
            bottles_available: false,
            bottles_enabled: false,
            bottles_error: None,
            bottles_loading: false,
            runtime_loading: false,
            runtime_operation_id: None,
            error: None,
        }
    }

    pub fn with_runtime_snapshot(
        runtimes: Vec<RuntimeStatus>,
        bottles_enabled: bool,
        bottles_available: bool,
        bottles: Vec<String>,
        bottles_default: Option<String>,
        bottles_error: Option<String>,
    ) -> Self {
        Self {
            runtimes,
            bottles,
            bottles_default,
            bottles_available,
            bottles_enabled,
            bottles_error,
            bottles_loading: false,
            runtime_loading: false,
            runtime_operation_id: None,
            error: None,
        }
    }

    pub fn runtimes(&self) -> &[RuntimeStatus] {
        &self.runtimes
    }

    pub fn bottles_enabled(&self) -> bool {
        self.bottles_enabled
    }

    pub fn bottles_available(&self) -> bool {
        self.bottles_available
    }

    pub fn bottles(&self) -> &[String] {
        &self.bottles
    }

    pub fn bottles_default(&self) -> Option<&str> {
        self.bottles_default.as_deref()
    }

    pub fn bottles_error(&self) -> Option<&str> {
        self.bottles_error.as_deref()
    }

    pub fn bottles_loading(&self) -> bool {
        self.bottles_loading
    }

    pub fn runtime_loading(&self) -> bool {
        self.runtime_loading
    }

    pub fn begin_runtime_operation(&mut self) {
        self.begin_runtime_operation_with_id(None);
    }

    pub fn begin_runtime_operation_with_id(&mut self, id: Option<OperationId>) {
        self.runtime_loading = true;
        self.runtime_operation_id = id;
        self.error = None;
    }

    pub fn finish_runtime_operation(&mut self, result: Result<(), String>) {
        self.runtime_loading = false;
        self.error = result.err();
    }

    pub fn runtime_operation_id(&self) -> Option<OperationId> {
        self.runtime_operation_id
    }

    pub fn clear_runtime_operation(&mut self, id: OperationId) {
        if self.runtime_operation_id == Some(id) {
            self.runtime_operation_id = None;
        }
    }

    pub fn runtime(&self, engine_type: &str) -> Option<&RuntimeStatus> {
        self.runtimes
            .iter()
            .filter(|runtime| runtime.engine_type.eq_ignore_ascii_case(engine_type))
            .max_by_key(|runtime| runtime.version.as_str())
    }

    pub fn can_select_bottles(&self) -> bool {
        self.bottles_enabled
            && self.bottles_available
            && !self.bottles_loading
            && !self.bottles.is_empty()
    }

    pub fn set_bottles_enabled(&mut self, enabled: bool) {
        self.bottles_enabled = enabled;
    }

    pub fn set_bottles_default(&mut self, bottle: Option<String>) {
        self.bottles_default = bottle;
    }

    pub fn begin_bottle_refresh(&mut self) {
        self.bottles_loading = true;
        self.bottles_error = None;
    }

    pub fn finish_bottle_refresh(&mut self, result: Result<Vec<String>, String>) {
        self.bottles_loading = false;
        match result {
            Ok(bottles) => {
                self.bottles = bottles;
                if self
                    .bottles_default
                    .as_ref()
                    .is_some_and(|default| !self.bottles.iter().any(|bottle| bottle == default))
                {
                    self.bottles_default = None;
                }
                self.bottles_error = None;
            }
            Err(error) => self.bottles_error = Some(error),
        }
    }

    pub fn nwjs_available(&self) -> bool {
        self.runtimes
            .iter()
            .any(|runtime| runtime.engine_type.eq_ignore_ascii_case("nwjs"))
    }

    pub fn mkxpz_available(&self) -> bool {
        self.runtimes
            .iter()
            .any(|runtime| runtime.engine_type.eq_ignore_ascii_case("mkxpz"))
    }
}
