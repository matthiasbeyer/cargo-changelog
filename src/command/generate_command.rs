use std::path::{Path, PathBuf};

use miette::IntoDiagnostic;

use crate::{cli::VersionSpec, config::Configuration, error::Error};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct GenerateCommand {
    version: VersionSpec,
}

impl crate::command::Command for GenerateCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> miette::Result<()> {
        let version_string = find_version_string(workdir, &self.version)?;
        log::debug!("Creating new directory for version '{}'", version_string);
        let release_dir = ensure_release_dir(workdir, config, &version_string)?;
        let unreleased_dir = workdir
            .join(config.fragment_dir())
            .join(crate::consts::UNRELEASED_DIR_NAME);

        log::info!("Computed unrelease dir: {}", unreleased_dir.display());
        log::info!("Computed release dir: {}", release_dir.display());

        let to_be_moved = std::fs::read_dir(&unreleased_dir)
            .map_err(Error::from)
            .into_diagnostic()?
            .into_iter()
            .map(|rdirentry| {
                rdirentry
                    .map(|de| de.path())
                    .map_err(Error::from)
                    .into_diagnostic()
            })
            .collect::<miette::Result<Vec<PathBuf>>>()?;

        for entry in to_be_moved {
            let entry_file_name = entry
                .file_name()
                .ok_or_else(|| miette::miette!("Apparently no file: {}", entry.display()))?;
            let destination = release_dir.join(entry_file_name);
            log::info!("Moving: {} -> {}", entry.display(), destination.display());
            std::fs::rename(entry, destination)
                .map_err(Error::from)
                .into_diagnostic()?;
        }

        Ok(())
    }
}

fn find_version_string(workdir: &Path, version: &VersionSpec) -> miette::Result<String> {
    use cargo_metadata::MetadataCommand;

    if let VersionSpec::Custom { custom } = version {
        Ok(custom.clone())
    } else {
        let metadata = MetadataCommand::new()
            .manifest_path(workdir.join("./Cargo.toml"))
            .exec()
            .map_err(Error::from)
            .into_diagnostic()?;

        let workspace_member_ids = &metadata.workspace_members;

        let versions = metadata
            .packages
            .iter()
            .filter(|pkg| workspace_member_ids.contains(&pkg.id))
            .map(|pkg| &pkg.version)
            .collect::<Vec<_>>();

        if versions.is_empty() {
            miette::bail!("No version found in Cargo.toml, that should never happen...")
        }

        let first = versions[0];
        let all_versions_same = versions.iter().all(|v| *v == first);
        if !all_versions_same {
            miette::bail!("Versions are not all the same in the workspace, cannot decide what you want to release!")
        }
        Ok(first.to_string())
    }
}

fn ensure_release_dir(
    workdir: &Path,
    config: &Configuration,
    version_string: &str,
) -> miette::Result<PathBuf> {
    let release_dir = workdir.join(config.fragment_dir()).join(version_string);
    std::fs::create_dir_all(&release_dir)
        .map_err(Error::from)
        .into_diagnostic()?;
    Ok(release_dir)
}
