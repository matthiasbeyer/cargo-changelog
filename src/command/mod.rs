use std::path::Path;

use crate::config::Configuration;

mod common;

mod add_command;
pub use self::add_command::AddCommand;

mod create_release_command;
pub use self::create_release_command::CreateReleaseCommand;

mod generate_changelog_command;
pub use self::generate_changelog_command::GenerateChangelogCommand;
pub use self::generate_changelog_command::VersionData;

mod show;
pub use self::show::Show;

mod verify_metadata_command;
pub use self::verify_metadata_command::VerifyMetadataCommand;

mod has;
pub use self::has::HasCommand;

pub trait Command {
    fn execute(
        self,
        workdir: &Path,
        config: &Configuration,
    ) -> Result<Option<std::process::ExitCode>, crate::error::Error>;
}
