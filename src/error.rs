use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DockerNotFound,
    DockerNotRunning,
    #[allow(dead_code)]
    JsonParseError(String),
    IoError(std::io::Error),
    TerminalError(String)
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DockerNotFound => write!(f, "Docker command not found. Please install Docker."),
            AppError::DockerNotRunning => write!(f, "Docker daemon is not running. Please start Docker."),
            AppError::JsonParseError(msg) => write!(f, "Failed to parse Docker stats: {msg}"),
            AppError::IoError(err) => write!(f, "IO error: {err}"),
            AppError::TerminalError(msg) => write!(f, "Terminal error: {msg}")
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AppError::DockerNotFound,
            std::io::ErrorKind::ConnectionRefused => AppError::DockerNotRunning,
            _ => AppError::IoError(err)
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
