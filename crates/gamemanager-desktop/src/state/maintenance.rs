use gamemanager_core::RuntimeStatus;

#[derive(Clone, Debug, Default)]
pub struct MaintenanceState {
    runtimes: Vec<RuntimeStatus>,
    pub error: Option<String>,
}

impl MaintenanceState {
    pub fn with_runtimes(runtimes: Vec<RuntimeStatus>) -> Self {
        Self {
            runtimes,
            error: None,
        }
    }

    pub fn runtimes(&self) -> &[RuntimeStatus] {
        &self.runtimes
    }
}
