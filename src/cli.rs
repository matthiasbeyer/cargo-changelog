use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap_complete::Shell;

use crate::config::GitSetting;
use crate::error::TextProviderError;
use crate::format::Format;

/// Get CLI args via `clap` while also handling when we are invoked as a cargo
/// subcommand
pub fn get_args() -> Args {
    // If we are invoked by cargo as `cargo changelog`, the second arg will
    // be "changelog". Remove it before passing args on to clap. If we are
    // not invoked as a cargo subcommand, it will not be part of args at all, so
    // it is safe to filter it out also in that case.
    let args = std::env::args_os().filter(|x| x != "changelog");
    Args::parse_from(args)
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize the repository for cargo-changelog
    Init,

    /// Create a new changelog fragment
    Add {
        #[clap(short, long, action = clap::ArgAction::Set, default_value_t = true)]
        interactive: bool,

        #[clap(short, long, action = clap::ArgAction::Set, default_value_t = true)]
        edit: bool,

        #[clap(short, long, value_enum, value_parser, default_value_t = Format::Toml)]
        format: Format,

        /// Read the changelog entry text from some path or stdin (via "-")
        #[clap(long, value_parser = text_provider_parser)]
        read: Option<TextProvider>,

        /// Set a header field to a specific value, non-interactively
        ///
        /// This expected a "key=value" argument, whereas the "key" part refers to a header field
        /// (e.g. "issue") and the "value" part is the value of that header field.
        ///
        /// E.G.: --set issue=123
        #[clap(long, value_parser = kv_value_parser)]
        set: Vec<KV>,

        /// Whether to execute a git command after creating the new entry.
        ///
        /// # Note
        ///
        /// If "commit" is given, and the "git_commit_message" setting in the configuration is NOT
        /// set, then the normal $GIT_EDITOR or $EDITOR will be spawned for the commit message.
        ///
        /// If "commit" is given and the "git_commit_message" setting is set, this message will be
        /// used.
        #[clap(long, value_enum, value_parser)]
        git: Option<GitSetting>,
    },

    /// Verify the metadata in existing changelog fragments
    VerifyMetadata,

    /// Use the current unreleased changelog fragments to generate the changelog for the next
    /// release
    #[clap(subcommand)]
    CreateRelease(VersionSpec),

    /// Generate the changelog file from the fragments marked for release
    GenerateChangelog {
        /// Also write "unreleased" stuff to the CHANGELOG.md file
        #[clap(long)]
        all: bool,

        #[clap(long, default_value_t = false)]
        allow_dirty: bool,
    },

    Show {
        #[clap(long)]
        format: Option<ShowFormat>,
        #[clap(subcommand)]
        selector: Option<Selector>,
    },
    /// Generation completions for the shell of your choice, available options:
    /// [bash, elvish, fish, powershell, zsh]
    GenerationCompletions {
        #[clap(value_parser)]
        shell: Shell,
    },
}

fn text_provider_parser(s: &str) -> Result<TextProvider, String> {
    if s == "-" {
        return Ok(TextProvider::Stdin);
    }

    let path = PathBuf::from(s);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    Ok(TextProvider::Path(path))
}

#[derive(Clone, Debug, PartialEq, Eq, getset::Getters)]
pub struct KV {
    #[getset(get = "pub")]
    key: String,
    #[getset(get = "pub")]
    value: String,
}

fn kv_value_parser(s: &str) -> Result<KV, String> {
    if s.chars().filter(|c| *c == '=').count() != 1 {
        Err(format!("Cannot parse as key-value: '{s}'"))
    } else {
        let (key, value) = s.split_once('=').unwrap(); // safe because above check
        Ok(KV {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextProvider {
    Stdin,
    Path(PathBuf),
}

impl TextProvider {
    pub fn read(&self) -> Result<String, TextProviderError> {
        use std::io::Read;

        match self {
            TextProvider::Stdin => {
                let mut buf = Vec::new();

                std::io::stdin().read_to_end(&mut buf)?;

                String::from_utf8(buf).map_err(TextProviderError::from)
            }
            TextProvider::Path(path) => {
                std::fs::read_to_string(path).map_err(TextProviderError::from)
            }
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum VersionSpec {
    Patch,
    Minor,
    Major,
    Custom {
        #[clap(value_parser)]
        custom: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum ShowFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Selector {
    /// Select unreleased changelogs
    Unreleased,

    /// Select changelogs with exact version
    Exact { exact: String },

    /// Select changelogs from version to version
    Range { from: String, until: String },
}
