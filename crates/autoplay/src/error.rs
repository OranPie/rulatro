use thiserror::Error;

#[derive(Debug, Error)]
pub enum AutoplayError {
    #[error("factory error: {0}")]
    Factory(String),
    #[error("run error: {0}")]
    Run(String),
    #[error("invalid action: {0}")]
    InvalidAction(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("serialize error: {0}")]
    Serialize(String),
}

impl From<std::io::Error> for AutoplayError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AutoplayError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value.to_string())
    }
}
