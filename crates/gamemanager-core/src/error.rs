use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("could not determine the platform data directory")]
    DataDirectoryUnavailable,
    #[error("invalid application path: {0}")]
    InvalidPath(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("engine error: {0}")]
    Engine(String),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("cover error: {0}")]
    Cover(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;

impl CoreError {
    pub fn database(error: impl std::fmt::Display) -> Self {
        Self::Database(error.to_string())
    }
}
