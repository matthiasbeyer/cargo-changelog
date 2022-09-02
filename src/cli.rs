use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

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
    Init,
    New {
        #[clap(short, long, action = clap::ArgAction::Set)]
        interactive: bool,

        #[clap(short, long, action = clap::ArgAction::Set, default_value_t = true)]
        edit: bool,

        #[clap(short, long, arg_enum, value_parser, default_value_t = Format::Yaml)]
        format: Format,

        #[clap(long, value_parser = text_provider_parser)]
        read: Option<TextProvider>,
    },

    #[clap(subcommand)]
    Generate(VersionSpec),

    Release,
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

#[derive(Debug, Subcommand)]
pub enum VersionSpec {
    Patch,
    Minor,
    Major,
    Custom {
        #[clap(value_parser)]
        custom: String,
    },
}
