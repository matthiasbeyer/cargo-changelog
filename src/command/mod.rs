use std::path::Path;

use crate::config::Configuration;

mod new_command;
pub use self::new_command::NewCommand;

pub trait Command {
    fn execute(self, workdir: &Path, config: &Configuration) -> miette::Result<()>;
}
