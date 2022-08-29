use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;

use crate::error::Error;
use crate::fragment::FragmentDataDesc;

pub const CONFIG_FILE_NAME: &'static str = ".changelog.toml";

#[derive(Debug, getset::Getters, serde::Deserialize, serde::Serialize)]
pub struct Configuration {
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

    /// The path of the template _inside the fragment directory_.
    ///
    /// By default: "template.md"
    ///
    /// ```rust
    /// assert_eq!(fragment_dir_default(), "template.md");
    /// ```
    #[getset(get = "pub")]
    #[serde(default = "template_path_default")]
    template_path: PathBuf,

    /// The path of the changelog file
    ///
    /// By default: "CHANGELOG.md"
    ///
    /// ```rust
    /// assert_eq!(changelog_default(), "CHANGELOG.md");
    /// ```
    #[getset(get = "pub")]
    #[serde(default = "changelog_default")]
    changelog: PathBuf,

    /// Whether to edit the data of a changelog entry in the editor
    edit_data: bool,
    /// Format to edit data in
    edit_format: EditFormat,
    #[getset(get = "pub")]
    header_fields: HashMap<String, FragmentDataDesc>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            add_version_date: true,
            fragment_dir: fragment_dir_default(),
            template_path: template_path_default(),
            changelog: changelog_default(),
            edit_data: true,
            edit_format: EditFormat::Yaml,
            header_fields: HashMap::new(),
        }
    }
}

pub fn fragment_dir_default() -> PathBuf {
    PathBuf::from(".changelogs")
}

pub fn template_path_default() -> PathBuf {
    PathBuf::from("template.md")
}

pub fn changelog_default() -> PathBuf {
    PathBuf::from("CHANGELOG.md")
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum EditFormat {
    Yaml,
    Toml,
}

impl std::str::FromStr for EditFormat {
    type Err = miette::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yaml" => Ok(Self::Yaml),
            "toml" => Ok(Self::Toml),
            fmt => Err(miette::miette!("Unknown edit format {}", fmt)),
        }
    }
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
