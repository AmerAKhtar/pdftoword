use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Internal engine failure: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
