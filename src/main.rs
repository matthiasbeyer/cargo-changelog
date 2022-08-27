use clap::{Subcommand, Parser};

mod config;
mod fragment;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New {
    },

    #[clap(subcommand)]
    Release(ReleaseSubcommand),

    Generate,

    #[clap(subcommand)]
    GenCompletions(GenCompletions),
}

#[derive(Subcommand)]
enum GenCompletions {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand)]
enum ReleaseSubcommand {
    Patch,
    Minor,
    Major,
    Custom {
        #[clap(value_parser)]
        custom: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init()?;

    let _args = Args::parse();
    println!("Hello, world!");

    Ok(())
}
