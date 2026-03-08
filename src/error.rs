use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    MissingHomeDir,
    InvalidDueDate,
    EmptyTitle,
    CorruptedData {
        path: PathBuf,
    },
    InvalidLocaleFile {
        path: PathBuf,
        source: serde_json::Error,
    },
    InvalidConfig {
        path: PathBuf,
    },
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

pub type AppResult<T> = Result<T, AppError>;
