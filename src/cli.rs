use clap::Parser;
use clap::Subcommand;

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
    New {},

    #[clap(subcommand)]
    Generate(VersionSpec),

    Release,
}

#[derive(Subcommand)]
pub enum VersionSpec {
    Patch,
    Minor,
    Major,
    Custom {
        #[clap(value_parser)]
        custom: String,
    },
}
