use std::path::{Path, PathBuf};

use crate::{cli::VersionSpec, config::Configuration, error::Error};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct GenerateCommand {
    version: VersionSpec,
}

impl crate::command::Command for GenerateCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> Result<(), Error> {
        let version_string = find_version_string(workdir, &self.version)?;
        log::debug!("Creating new directory for version '{}'", version_string);
        let release_dir = ensure_release_dir(workdir, config, &version_string)?;
        let unreleased_dir = workdir
            .join(config.fragment_dir())
            .join(crate::consts::UNRELEASED_DIR_NAME);

        log::info!("Computed unrelease dir: {}", unreleased_dir.display());
        log::info!("Computed release dir: {}", release_dir.display());

        let to_be_moved = std::fs::read_dir(&unreleased_dir)?
            .into_iter()
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

        Ok(())
    }
}

fn find_version_string(workdir: &Path, version: &VersionSpec) -> Result<String, Error> {
    use cargo_metadata::MetadataCommand;

    if let VersionSpec::Custom { custom } = version {
        Ok(custom.clone())
    } else {
        let metadata = MetadataCommand::new()
            .manifest_path(workdir.join("./Cargo.toml"))
            .exec()?;

        let workspace_member_ids = &metadata.workspace_members;

        let versions = metadata
            .packages
            .iter()
            .filter(|pkg| workspace_member_ids.contains(&pkg.id))
            .map(|pkg| &pkg.version)
            .collect::<Vec<_>>();

        if versions.is_empty() {
            return Err(Error::NoVersionInCargoToml);
        }

        let first = versions[0];
        let all_versions_same = versions.iter().all(|v| *v == first);
        if !all_versions_same {
            return Err(Error::WorkspaceVersionsNotEqual);
        }
        Ok(first.to_string())
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
