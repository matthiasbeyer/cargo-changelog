use std::io;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::generate;
use cli::Args;
use miette::IntoDiagnostic;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod cli;
mod command;
mod config;
mod consts;
mod error;
mod format;
mod fragment;
mod selector;
mod template;
mod util;

use crate::cli::Command;
use crate::command::Command as _;
use crate::error::Error;

fn main() -> miette::Result<std::process::ExitCode> {
    let args = cli::get_args();

    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(args.verbose.tracing_level_filter().into())
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    let cwd = std::env::current_dir()
        .map_err(Error::from)
        .into_diagnostic()?;

    let repository = git2::Repository::open(cwd)
        .map_err(Error::from)
        .into_diagnostic()?;

    let repo_workdir_path = repository
        .workdir()
        .ok_or(Error::NoWorkTree)
        .into_diagnostic()?
        .to_path_buf();

    if let Command::Init = args.command {
        return init(repo_workdir_path).map(|_| std::process::ExitCode::SUCCESS);
    }

    let config = crate::config::load(&repo_workdir_path)?;

    if !config.fragment_dir().exists() {
        let fragment_dir_path = {
            let mut fragment_dir_path = repo_workdir_path.clone();
            fragment_dir_path.push(config.fragment_dir());
            fragment_dir_path
        };

        std::fs::create_dir_all(fragment_dir_path)
            .map_err(Error::from)
            .into_diagnostic()?;
    }

    let opt_exit_code = match args.command {
        Command::Init => unreachable!(), // reached above

        Command::Add {
            interactive,
            edit,
            format,
            read,
            set,
            git,
        } => crate::command::AddCommand::builder()
            .interactive(interactive)
            .edit(edit)
            .format(format)
            .text(read)
            .set(set)
            .git(git)
            .build()
            .execute(&repo_workdir_path, &config)?,

        Command::VerifyMetadata => crate::command::VerifyMetadataCommand::builder()
            .build()
            .execute(&repo_workdir_path, &config)?,

        Command::CreateRelease(version) => crate::command::CreateReleaseCommand::builder()
            .version(version)
            .build()
            .execute(&repo_workdir_path, &config)?,

        Command::GenerateChangelog { all, allow_dirty } => {
            crate::command::GenerateChangelogCommand::builder()
                .repository(repository)
                .all(all)
                .allow_dirty(allow_dirty)
                .build()
                .execute(&repo_workdir_path, &config)?
        }

        Command::Show { format, selector } => crate::command::Show::builder()
            .format(format)
            .selector(selector)
            .build()
            .execute(&repo_workdir_path, &config)?,
        Command::GenerationCompletions { shell } => {
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "cargo-changelog", &mut io::stdout());
            None
        }

        Command::Has { format, selector } => crate::command::HasCommand::builder()
            .format(format)
            .selector(selector)
            .build()
            .execute(&repo_workdir_path, &config)?,
    };

    Ok(opt_exit_code.unwrap_or(std::process::ExitCode::SUCCESS))
}

fn init(repo_workdir_path: PathBuf) -> miette::Result<()> {
    use std::io::Write;

    let unreleased_dir_path = repo_workdir_path
        .join(crate::config::fragment_dir_default())
        .join("unreleased");

    std::fs::create_dir_all(&unreleased_dir_path)
        .map_err(Error::from)
        .into_diagnostic()?;

    std::fs::File::create(unreleased_dir_path.join(".gitkeep"))
        .map_err(Error::from)
        .into_diagnostic()?;

    let mut config_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .append(false)
        .write(true)
        .open(repo_workdir_path.join(crate::config::CONFIG_FILE_DEFAULT_NAME))
        .map_err(Error::from)
        .into_diagnostic()?;

    write!(&mut config_file, "{}", crate::config::DEFAULT_CONFIG)
        .map_err(Error::from)
        .into_diagnostic()?;

    config_file
        .sync_all()
        .map_err(Error::from)
        .into_diagnostic()?;

    let mut template_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .append(false)
        .write(true)
        .open({
            repo_workdir_path
                .join(crate::config::fragment_dir_default())
                .join(crate::config::template_path_default())
        })
        .map_err(Error::from)
        .into_diagnostic()?;

    write!(&mut template_file, "{}", crate::consts::DEFAULT_TEMPLATE)
        .map_err(Error::from)
        .into_diagnostic()?;

    template_file
        .sync_all()
        .map_err(Error::from)
        .into_diagnostic()?;

    let existing_changelog = repo_workdir_path.join("CHANGELOG.md");

    if existing_changelog.exists() {
        let suffix_path = repo_workdir_path
            .join(crate::config::fragment_dir_default())
            .join("suffix.md");

        std::fs::rename(existing_changelog, &suffix_path)
            .map_err(Error::from)
            .into_diagnostic()?;

        println!(
            "Found an existing CHANGELOG.md, moved it to {}",
            suffix_path.display()
        );
    }

    Ok(())
}
