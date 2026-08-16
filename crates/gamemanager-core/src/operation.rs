use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};

use futures_util::{StreamExt, stream};
use tokio::sync::watch;

use crate::Result;

static NEXT_OPERATION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OperationId(u64);

impl OperationId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn value(self) -> u64 {
        self.0
    }
    fn next() -> Self {
        Self(NEXT_OPERATION_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationStage {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationProgress {
    pub id: OperationId,
    pub stage: String,
    pub percent: Option<u8>,
    pub state: OperationStage,
}

impl OperationProgress {
    pub fn new(id: OperationId, stage: impl Into<String>, percent: Option<u8>) -> Self {
        Self {
            id,
            stage: stage.into(),
            percent: percent.map(|value| value.min(100)),
            state: OperationStage::Running,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum OperationOutcome<T> {
    Completed(T),
    Cancelled,
}

type BoxedResult<T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'static>>;

struct CompletionSignal {
    progress: OperationProgress,
    sender: watch::Sender<Option<OperationProgress>>,
}

pub struct Operation<T> {
    id: OperationId,
    progress: Vec<OperationProgress>,
    completion: Option<CompletionSignal>,
    result: Option<BoxedResult<T>>,
}

impl<T: Send + 'static> Operation<T> {
    pub fn from_steps<I, S, F>(steps: I, result: F) -> Self
    where
        I: IntoIterator<Item = (S, u8)>,
        S: Into<String>,
        F: Future<Output = Result<T>> + Send + 'static,
    {
        let id = OperationId::next();
        let progress = steps
            .into_iter()
            .map(|(stage, percent)| OperationProgress::new(id, stage, Some(percent)))
            .collect();
        Self {
            id,
            progress,
            completion: None,
            result: Some(Box::pin(result)),
        }
    }

    pub fn from_future<F>(stage: impl Into<String>, result: F) -> Self
    where
        F: Future<Output = Result<T>> + Send + 'static,
    {
        let id = OperationId::next();
        let stage = stage.into();
        let progress = vec![OperationProgress::new(id, stage.clone(), Some(0))];
        let (sender, _) = watch::channel(None);
        Self {
            id,
            progress,
            completion: Some(CompletionSignal {
                progress: OperationProgress::new(id, stage, Some(100)),
                sender,
            }),
            result: Some(Box::pin(result)),
        }
    }

    pub fn id(&self) -> OperationId {
        self.id
    }

    pub fn progress(&self) -> futures_util::stream::BoxStream<'static, OperationProgress> {
        let initial = stream::iter(self.progress.clone());
        let Some(completion) = self.completion.as_ref() else {
            return initial.boxed();
        };
        let mut receiver = completion.sender.subscribe();
        initial
            .chain(
                stream::once(async move {
                    receiver
                        .wait_for(|progress| progress.is_some())
                        .await
                        .ok()
                        .and_then(|progress| (*progress).clone())
                })
                .filter_map(|progress| async move { progress }),
            )
            .boxed()
    }

    pub async fn into_future(mut self) -> Result<T> {
        let result = self
            .result
            .take()
            .expect("operation future already consumed")
            .await;
        if let Some(completion) = self.completion {
            let mut progress = completion.progress;
            progress.state = if result.is_ok() {
                OperationStage::Completed
            } else {
                OperationStage::Failed
            };
            let _ = completion.sender.send(Some(progress));
        }
        result
    }

    pub fn cancel(self) -> OperationOutcome<T> {
        OperationOutcome::Cancelled
    }
}
