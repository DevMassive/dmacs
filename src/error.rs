use thiserror::Error;

#[derive(Error, Debug)]
pub enum DmacsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Editor error: {0}")]
    Editor(String),
    #[error("Document error: {0}")]
    Document(String),
    #[error("Backup not found for {0}")]
    BackupNotFound(String),
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, DmacsError>;
