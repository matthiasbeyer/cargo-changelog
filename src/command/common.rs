use std::path::Path;

pub fn get_version_from_path(path: &Path) -> miette::Result<Option<semver::Version>> {
    path.components()
        .find_map(|comp| match comp {
            std::path::Component::Normal(comp) => {
                let s = comp
                    .to_str()
                    .ok_or_else(|| miette::miette!("UTF8 Error in path: {:?}", comp));

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
