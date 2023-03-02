use std::path::Path;

use crate::{
    cli::VersionSpec,
    error::{Error, VersionError},
};

pub fn get_version_from_path(path: &Path) -> Result<Option<semver::Version>, VersionError> {
    path.components()
        .find_map(|comp| match comp {
            std::path::Component::Normal(comp) => {
                let s = comp
                    .to_str()
                    .ok_or_else(|| VersionError::Utf8(path.to_path_buf()));

                match s {
                    Err(e) => Some(Err(e)),
                    Ok(s) => {
                        log::debug!("Parsing '{}' as semver", s);
                        match semver::Version::parse(s) {
                            Err(_) => None,
                            Ok(semver) => Some(Ok(semver)),
                        }
                    }
                }
            }
            _ => None,
        })
        .transpose()
}

pub fn find_version_string(workdir: &Path, version: &VersionSpec) -> Result<String, Error> {
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
