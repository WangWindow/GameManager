use std::collections::BTreeMap;

use gamemanager_core::{OperationId, OperationProgress, OperationStage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationView {
    pub id: OperationId,
    pub label: String,
    pub percent: Option<u8>,
    pub state: OperationStage,
}

#[derive(Clone, Debug, Default)]
pub struct OperationState {
    entries: BTreeMap<u64, OperationView>,
}

impl OperationState {
    pub fn apply(&mut self, progress: OperationProgress) {
        self.entries.insert(
            progress.id.value(),
            OperationView {
                id: progress.id,
                label: progress.stage,
                percent: progress.percent,
                state: progress.state,
            },
        );
    }

    pub fn get(&self, id: OperationId) -> Option<&OperationView> {
        self.entries.get(&id.value())
    }
}
