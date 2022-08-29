use std::path::Path;

use crate::config::Configuration;

mod new_command;
pub use self::new_command::NewCommand;

mod generate_command;
pub use self::generate_command::GenerateCommand;

mod release_command;
pub use self::release_command::ReleaseCommand;

pub trait Command {
    fn execute(self, workdir: &Path, config: &Configuration) -> miette::Result<()>;
}
