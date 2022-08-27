use std::collections::HashMap;
use std::path::PathBuf;

use miette::IntoDiagnostic;

use crate::error::Error;

#[derive(Debug, serde::Deserialize)]
pub struct Configuration {
    changelog_header: String,
    version_prefix: String,
    add_version_date: bool,

    entry_template: PathBuf,
    entry_data: Vec<FragmentDataDescription>,

    group_by: Option<FragmentDataDescriptionName>,
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
pub fn load() -> miette::Result<Configuration> {
    let cwd = std::env::current_dir()
        .map_err(Error::from)
        .into_diagnostic()?;

    let repo = git2::Repository::open(cwd)
        .map_err(Error::from)
        .into_diagnostic()?;

    let workdir_path = repo
        .workdir()
        .ok_or_else(|| Error::NoWorkTree)
        .into_diagnostic()?
        .to_path_buf();

    let changelog_config_path = {
        let mut cfg_path = workdir_path;
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
