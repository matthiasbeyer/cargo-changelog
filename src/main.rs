mod cli;
mod config;
mod error;
mod fragment;

use crate::cli::Command;
use crate::cli::VersionSpec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init()?;

    let _config = crate::config::load()?;

    let args = cli::get_args();
    match args.command {
        Command::New { .. } => {}

        Command::Generate(VersionSpec::Patch) => {}

        Command::Generate(VersionSpec::Minor) => {}

        Command::Generate(VersionSpec::Major) => {}

        Command::Generate(VersionSpec::Custom { custom: _ }) => {}

        Command::Release => {}
    }

    Ok(())
}
