use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("could not determine the platform data directory")]
    DataDirectoryUnavailable,
    #[error("invalid application path: {0}")]
    InvalidPath(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
