use std::path::Path;
use std::path::PathBuf;

use indexmap::IndexMap;
use miette::IntoDiagnostic;

use crate::error::Error;
use crate::fragment::FragmentDataDesc;

pub const CONFIG_FILE_NAMES: &[&str] = &[".changelog.toml", "changelog.toml"];
pub const CONFIG_FILE_DEFAULT_NAME: &str = CONFIG_FILE_NAMES[1];
pub const DEFAULT_CONFIG: &str = include_str!("../assets/default_config.toml");

#[derive(Debug, getset::Getters, getset::CopyGetters, serde::Deserialize, serde::Serialize)]
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

    /// Set whether and how to use git after creating a new entry
    ///
    /// Possible values are "add" or "commit" (or none, which is default).
    ///
    /// "add" means only git-add the newly created file
    /// "commit" means commit the newly created file as well, with a default message
    ///
    #[getset(get = "pub")]
    git: Option<GitSetting>,

    /// The commit message to use if `git = "commit"` is set.
    ///
    /// Can also be set if `git = "add"` is configured, because the CLI might override this
    /// setting.
    #[getset(get = "pub")]
    git_commit_message: Option<String>,

    /// Use the --signoff flag in case of `git = "commit"`
    #[getset(get_copy = "pub")]
    git_commit_signoff: bool,

    #[getset(get = "pub")]
    header_fields: IndexMap<String, FragmentDataDesc>,
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
#[serde(rename_all = "lowercase")]
pub enum EditFormat {
    Toml,
}

impl std::str::FromStr for EditFormat {
    type Err = miette::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "toml" => Ok(Self::Toml),
            fmt => Err(miette::miette!("Unknown edit format {}", fmt)),
        }
    }
}

/// Load the configuration from the repository
pub fn load(repo_workdir_path: &Path) -> miette::Result<Configuration> {
    let mut changelog_config_path = None;
    for config_path in CONFIG_FILE_NAMES {
        let check_path = {
            let mut cfg_path = repo_workdir_path.to_path_buf();
            cfg_path.push(config_path);
            cfg_path
        };

        if check_path.exists() {
            changelog_config_path = Some(check_path);
            break;
        }
    }

    let Some(changelog_config_path) = changelog_config_path else {
        miette::bail!(Error::ConfigDoesNotExist)
    };

    let config = std::fs::read_to_string(changelog_config_path)
        .map_err(Error::from)
        .into_diagnostic()?;

    toml::from_str(&config)
        .map_err(Error::from)
        .into_diagnostic()
}

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, serde::Deserialize, serde::Serialize,
)]
pub enum GitSetting {
    Add,
    Commit,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_default_config_deserializes_to_configuration() {
        let config = toml::from_str::<super::Configuration>(super::DEFAULT_CONFIG);
        assert!(config.is_ok(), "Not ok: {:?}", config.unwrap_err());
    }

    #[test]
    fn test_default_config_has_default_fragment_dir() {
        let config: super::Configuration = toml::from_str(super::DEFAULT_CONFIG).unwrap();
        assert_eq!(
            *config.fragment_dir(),
            super::fragment_dir_default(),
            "Fragment dir from default config is not {}",
            super::fragment_dir_default().display()
        );
    }

    #[test]
    fn test_default_config_has_default_template_path() {
        let config: super::Configuration = toml::from_str(super::DEFAULT_CONFIG).unwrap();
        assert_eq!(
            *config.template_path(),
            super::template_path_default(),
            "Template path from default config is not {}",
            super::fragment_dir_default().display()
        );
    }
}
