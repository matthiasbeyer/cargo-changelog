use std::path::PathBuf;

use miette::IntoDiagnostic;

mod cli;
mod command;
mod config;
mod consts;
mod error;
mod format;
mod fragment;

use crate::cli::Command;
use crate::cli::VersionSpec;
use crate::command::Command as _;
use crate::error::Error;

fn main() -> miette::Result<()> {
    env_logger::try_init().into_diagnostic()?;

    let args = cli::get_args();

    let cwd = std::env::current_dir()
        .map_err(Error::from)
        .into_diagnostic()?;

    let repository = git2::Repository::open(cwd)
        .map_err(Error::from)
        .into_diagnostic()?;

    let repo_workdir_path = repository
        .workdir()
        .ok_or_else(|| Error::NoWorkTree)
        .into_diagnostic()?
        .to_path_buf();

    if let Command::Init = args.command {
        return init(repo_workdir_path);
    }

    let config = crate::config::load(&repo_workdir_path)?;

    if !config.fragment_dir().exists() {
        let fragment_dir_path = {
            let mut fragment_dir_path = repo_workdir_path.to_path_buf();
            fragment_dir_path.push(config.fragment_dir());
            fragment_dir_path
        };

        std::fs::create_dir_all(fragment_dir_path)
            .map_err(Error::from)
            .into_diagnostic()?;
    }

    match args.command {
        Command::Init => unreachable!(), // reached above

        Command::New {
            interactive,
            edit,
            format,
            read,
        } => crate::command::NewCommand::builder()
            .interactive(interactive)
            .edit(edit)
            .format(format)
            .text(read)
            .build()
            .execute(&repo_workdir_path, &config),

        Command::Generate(version) => crate::command::GenerateCommand::builder()
            .version(version)
            .build()
            .execute(&repo_workdir_path, &config),

        Command::Release => {
            unimplemented!()
        }
    }
}

fn init(repo_workdir_path: PathBuf) -> miette::Result<()> {
    use std::io::Write;

    std::fs::create_dir_all({
        repo_workdir_path
            .join(crate::config::fragment_dir_default())
            .join("unreleased")
    })
    .map_err(Error::from)
    .into_diagnostic()?;

    let default_config = crate::config::Configuration::default();
    let default_config_str = toml::to_string(&default_config).unwrap(); // cannot happen

    let mut config_file = std::fs::OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open(repo_workdir_path.join(crate::config::CONFIG_FILE_NAME))
        .map_err(Error::from)
        .into_diagnostic()?;

    write!(&mut config_file, "{}", default_config_str)
        .map_err(Error::from)
        .into_diagnostic()?;

    config_file
        .sync_all()
        .map_err(Error::from)
        .into_diagnostic()?;

    let mut template_file = std::fs::OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open({
            repo_workdir_path
                .join(default_config.fragment_dir())
                .join(default_config.template_path())
        })
        .map_err(Error::from)
        .into_diagnostic()?;

    write!(&mut template_file, "{}", crate::consts::DEFAULT_TEMPLATE)
        .map_err(Error::from)
        .into_diagnostic()?;

    template_file
        .sync_all()
        .map_err(Error::from)
        .into_diagnostic()
}
