use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum EccError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Claude config directory not found at {0}")]
    ClaudeDirNotFound(String),

    #[error("Hook validation: {0}")]
    HookValidation(String),

    #[error("Cross-reference broken: {0} -> {1}")]
    CrossReference(String, String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("No sessions found in project")]
    NoSessionsFound,

    #[error("{0}")]
    Other(String),
}

pub type EccResult<T> = Result<T, EccError>;
