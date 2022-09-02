use std::path::Path;

use crate::error::VersionError;

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
