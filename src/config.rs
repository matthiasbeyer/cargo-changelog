use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;

use crate::error::Error;

#[derive(Debug, getset::Getters, serde::Deserialize)]
pub struct Configuration {
    changelog_header: String,
    version_prefix: String,
    add_version_date: bool,

    /// Directory name where fragments will be stored
    ///
    /// By default: ".changelogs"
    ///
    /// ```rust
    /// assert_eq!(fragment_dir_default(), ".changelogs");
    /// ```
    #[getset(get = "pub")]
    #[serde(default = "fragment_dir_default")]
    fragment_dir: PathBuf,

    entry_template: PathBuf,
    entry_data: Vec<FragmentDataDescription>,

    group_by: Option<FragmentDataDescriptionName>,
}

fn fragment_dir_default() -> PathBuf {
    PathBuf::from(".changelogs")
}

#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct FragmentDataDescriptionName(String);

#[derive(Debug, serde::Deserialize)]
pub struct FragmentDataDescription {
    key: FragmentDataDescriptionName,
    required: bool,
    default_value: Option<String>,
    value: FragmentDataValueType,
}

#[derive(Debug, serde::Deserialize)]
pub enum FragmentDataValueType {
    Bool,
    Int,
    String,
    Map(HashMap<String, FragmentDataDescription>),
}

/// Load the configuration from the repository
pub fn load(repo_workdir_path: &Path) -> miette::Result<Configuration> {
    let changelog_config_path = {
        let mut cfg_path = repo_workdir_path.to_path_buf();
        cfg_path.push(".changelog_config.toml");
        cfg_path
    };

    if !changelog_config_path.exists() {
        miette::bail!(Error::ConfigDoesNotExist(changelog_config_path))
    }

    let config = std::fs::read_to_string(changelog_config_path)
        .map_err(Error::from)
        .into_diagnostic()?;

    toml::from_str(&config)
        .map_err(Error::from)
        .into_diagnostic()
}
