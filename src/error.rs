use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO")]
    Io(#[from] std::io::Error),

    #[error("UTF8 error")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("git error")]
    Git(#[from] git2::Error),

    #[error("TOML deserialization error")]
    Toml(#[from] toml::de::Error),

    #[error("YAML error")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Time formatting error")]
    TimeFormat(#[from] time::error::Format),

    #[error("Error getting cargo meta information")]
    Cargo(#[from] cargo_metadata::Error),

    #[error("Repository has no worktree")]
    NoWorkTree,

    #[error("Configuration file does not exist: {0}")]
    ConfigDoesNotExist(PathBuf),
}
