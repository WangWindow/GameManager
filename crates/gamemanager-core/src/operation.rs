use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};

use futures_util::{StreamExt, stream};
use tokio::sync::broadcast;

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

/// Allows an asynchronous operation to publish intermediate progress while
/// its result future is running.
#[derive(Clone)]
pub struct OperationReporter {
    id: OperationId,
    stage: String,
    sender: broadcast::Sender<OperationProgress>,
}

impl OperationReporter {
    pub fn report(&self, percent: Option<u8>) {
        self.report_stage(self.stage.clone(), percent);
    }

    pub fn report_stage(&self, stage: impl Into<String>, percent: Option<u8>) {
        let _ = self
            .sender
            .send(OperationProgress::new(self.id, stage, percent));
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
}

pub struct Operation<T> {
    id: OperationId,
    progress: Vec<OperationProgress>,
    sender: broadcast::Sender<OperationProgress>,
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
        let (sender, _) = broadcast::channel(128);
        let progress = steps
            .into_iter()
            .map(|(stage, percent)| OperationProgress::new(id, stage, Some(percent)))
            .collect();
        Self {
            id,
            progress,
            sender,
            completion: None,
            result: Some(Box::pin(result)),
        }
    }

    pub fn from_future<F>(stage: impl Into<String>, result: F) -> Self
    where
        F: Future<Output = Result<T>> + Send + 'static,
    {
        Self::from_future_with_progress(stage, move |_| result)
    }

    pub fn from_future_with_progress<F, Fut>(stage: impl Into<String>, build: F) -> Self
    where
        F: FnOnce(OperationReporter) -> Fut,
        Fut: Future<Output = Result<T>> + Send + 'static,
    {
        let id = OperationId::next();
        let stage = stage.into();
        let (sender, _) = broadcast::channel(128);
        let reporter = OperationReporter {
            id,
            stage: stage.clone(),
            sender: sender.clone(),
        };
        Self {
            id,
            progress: vec![OperationProgress::new(id, stage.clone(), Some(0))],
            sender,
            completion: Some(CompletionSignal {
                progress: OperationProgress::new(id, stage, Some(100)),
            }),
            result: Some(Box::pin(build(reporter))),
        }
    }

    pub fn id(&self) -> OperationId {
        self.id
    }

    pub fn progress(&self) -> futures_util::stream::BoxStream<'static, OperationProgress> {
        let initial = stream::iter(self.progress.clone());
        if self.completion.is_none() {
            return initial.boxed();
        }
        let receiver = self.sender.subscribe();
        let updates = stream::unfold(receiver, |mut receiver| async move {
            loop {
                match receiver.recv().await {
                    Ok(progress) => return Some((progress, receiver)),
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        });
        initial.chain(updates).boxed()
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
            let _ = self.sender.send(progress);
        }
        result
    }

    pub fn cancel(self) -> OperationOutcome<T> {
        OperationOutcome::Cancelled
    }
}
