use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO")]
    Io(#[from] std::io::Error),

    #[error("git error")]
    Git(#[from] git2::Error),

    #[error("TOML deserialization error")]
    Toml(#[from] toml::de::Error),

    #[error("Repository has no worktree")]
    NoWorkTree,

    #[error("Configuration file does not exist: {0}")]
    ConfigDoesNotExist(PathBuf),
}
