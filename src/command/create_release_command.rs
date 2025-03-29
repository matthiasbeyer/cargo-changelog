use std::path::{Path, PathBuf};

use crate::{
    cli::VersionSpec, command::common::find_version_string, config::Configuration, error::Error,
};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct CreateReleaseCommand {
    version: VersionSpec,
}

impl crate::command::Command for CreateReleaseCommand {
    fn execute(
        self,
        workdir: &Path,
        config: &Configuration,
    ) -> Result<Option<std::process::ExitCode>, Error> {
        let version_string = find_version_string(workdir, &self.version)?;
        log::debug!("Creating new directory for version '{}'", version_string);
        let release_dir = ensure_release_dir(workdir, config, &version_string)?;
        let unreleased_dir = workdir
            .join(config.fragment_dir())
            .join(crate::consts::UNRELEASED_DIR_NAME);

        log::info!("Computed unrelease dir: {}", unreleased_dir.display());
        log::info!("Computed release dir: {}", release_dir.display());

        let to_be_moved = std::fs::read_dir(&unreleased_dir)?
            .map(|rdirentry| rdirentry.map(|de| de.path()).map_err(Error::from))
            .filter(|rpb| match rpb {
                Ok(pb) => !pb.ends_with(".gitkeep"),
                Err(_) => true,
            })
            .collect::<Result<Vec<PathBuf>, _>>()?;

        for entry in to_be_moved {
            let entry_file_name = entry
                .file_name()
                .ok_or_else(|| Error::NotAFile(entry.to_path_buf()))?;
            let destination = release_dir.join(entry_file_name);
            log::info!("Moving: {} -> {}", entry.display(), destination.display());
            std::fs::rename(entry, destination)?;
        }

        Ok(None)
    }
}

fn ensure_release_dir(
    workdir: &Path,
    config: &Configuration,
    version_string: &str,
) -> Result<PathBuf, Error> {
    let release_dir = workdir.join(config.fragment_dir()).join(version_string);
    std::fs::create_dir_all(&release_dir)?;
    Ok(release_dir)
}
