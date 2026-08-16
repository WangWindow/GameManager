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
    latest: Option<OperationId>,
}

impl OperationState {
    pub fn apply(&mut self, progress: OperationProgress) {
        self.latest = Some(progress.id);
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

    pub fn current(&self) -> Option<&OperationView> {
        self.latest.and_then(|id| self.get(id))
    }

    pub fn clear(&mut self, id: OperationId) {
        self.entries.remove(&id.value());
        if self.latest == Some(id) {
            self.latest = self
                .entries
                .keys()
                .next_back()
                .copied()
                .map(OperationId::new);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_operation_tracks_latest_progress_and_can_be_cleared() {
        let first = OperationId::new(1);
        let second = OperationId::new(2);
        let mut state = OperationState::default();

        state.apply(OperationProgress::new(first, "first", Some(10)));
        state.apply(OperationProgress::new(second, "second", Some(20)));
        assert_eq!(state.current().map(|operation| operation.id), Some(second));

        state.clear(second);
        assert_eq!(state.current().map(|operation| operation.id), Some(first));
        state.clear(first);
        assert!(state.current().is_none());
    }
}
