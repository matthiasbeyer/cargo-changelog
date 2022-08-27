use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;

use crate::error::Error;

pub const CONFIG_FILE_NAME: &'static str = ".changelog.toml";

#[derive(Debug, getset::Getters, serde::Deserialize, serde::Serialize)]
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

    group_by: Option<FragmentDataDescriptionName>,

    entry_template: PathBuf,
    entry_data: Vec<FragmentDataDescription>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            changelog_header: "# Changelog".to_string(),
            version_prefix: "v".to_string(),
            add_version_date: true,

            fragment_dir: PathBuf::from("fragments"),

            entry_template: PathBuf::from("entry_template.md"),
            entry_data: vec![FragmentDataDescription {
                key: FragmentDataDescriptionName("type".to_string()),
                required: true,
                default_value: None,
                value: FragmentDataValueType::String,
            }],

            group_by: Some(FragmentDataDescriptionName("type".to_string())),
        }
    }
}

pub fn fragment_dir_default() -> PathBuf {
    PathBuf::from(".changelogs")
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct FragmentDataDescriptionName(String);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FragmentDataDescription {
    key: FragmentDataDescriptionName,
    required: bool,
    default_value: Option<String>,
    value: FragmentDataValueType,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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
        cfg_path.push(CONFIG_FILE_NAME);
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
