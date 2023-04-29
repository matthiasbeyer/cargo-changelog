use std::io::Write;
use std::{collections::HashMap, io::BufReader, path::Path};

use crate::{config::Configuration, error::Error, fragment::Fragment};

#[derive(typed_builder::TypedBuilder)]
pub struct ReleaseCommand {
    repository: git2::Repository,
    all: bool,
    allow_dirty: bool,
}

impl std::fmt::Debug for ReleaseCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReleaseCommand")
            .field("repository", &self.repository.workdir())
            .field("all", &self.all)
            .field("allow_dirty", &self.allow_dirty)
            .finish_non_exhaustive()
    }
}

impl crate::command::Command for ReleaseCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> Result<(), Error> {
        if crate::util::repo_is_dirty(&self.repository) && !self.allow_dirty {
            return Err(Error::GitRepoDirty);
        }

        let template_path = workdir
            .join(config.fragment_dir())
            .join(config.template_path());
        let template_source = std::fs::read_to_string(template_path)?;
        let template = crate::template::new_handlebars(&template_source)?;

        let suffix_path = workdir.join(config.fragment_dir()).join("suffix.md");
        let suffix = match std::fs::read_to_string(&suffix_path) {
            Ok(suffix) => Some(suffix),
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        // We don't want to spam the user for something they don't use
                        log::trace!(
                            "Did not find {}, not appending suffix",
                            suffix_path.display()
                        )
                    }
                    _ => {
                        log::error!(
                            "Could not read suffix file at {}: {err}",
                            suffix_path.display()
                        );
                    }
                }
                None
            }
        };

        let template_data =
            generate_template_data(load_release_files(workdir, config, self.all), suffix)?;

        let changelog_contents =
            template.render(crate::consts::INTERNAL_TEMPLATE_NAME, &template_data)?;
        log::debug!("Rendered successfully");

        let changelog_file_path = workdir.join(config.changelog());
        log::debug!(
            "Writing changelog file now: {}",
            changelog_file_path.display()
        );
        let mut changelog_file = std::fs::OpenOptions::new()
            .create(true)
            .append(false)
            .truncate(true)
            .write(true)
            .open(changelog_file_path)?;

        write!(changelog_file, "{changelog_contents}")?;
        changelog_file.sync_all()?;
        Ok(())
    }
}

fn load_release_files(
    workdir: &Path,
    config: &Configuration,
    all: bool,
) -> impl Iterator<Item = Result<(Option<semver::Version>, Fragment), Error>> {
    walkdir::WalkDir::new(workdir.join(config.fragment_dir()))
        .follow_links(false)
        .max_open(100)
        .same_file_system(true)
        .into_iter()
        .filter_map(|rde| match rde {
            Err(e) => Some(Err(e)),
            Ok(de) => {
                if de.file_type().is_file() {
                    if de.path().ends_with("template.md") || de.path().ends_with(".gitkeep") {
                        None
                    } else {
                        log::debug!("Considering: {:?}", de);
                        Some(Ok(de))
                    }
                } else {
                    None
                }
            }
        })
        .filter_map(move |rde| {
            let de = match rde {
                Err(e) => return Some(Err(Error::from(e))),
                Ok(de) => de,
            };

            let version = match crate::command::common::get_version_from_path(de.path()) {
                Err(e) => return Some(Err(Error::from(e))),
                Ok(None) => {
                    if all {
                        None
                    } else {
                        return None;
                    }
                }
                Ok(Some(version)) => Some(version),
            };

            let fragment = std::fs::OpenOptions::new()
                .read(true)
                .create(false)
                .write(false)
                .open(de.path())
                .map_err(Error::from)
                .map(BufReader::new)
                .and_then(|mut reader| {
                    Fragment::from_reader(&mut reader)
                        .map_err(|e| Error::FragmentError(e, de.path().to_path_buf()))
                });

            match fragment {
                Err(e) => Some(Err(e)),
                Ok(fragment) => Some(Ok((version, fragment))),
            }
        })
}

/// The data sent to the handlebars template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, getset::Getters)]
pub struct TemplateData {
    versions: Vec<VersionData>,
    suffix: Option<String>,
}

/// Helper type for storing version associated with Fragments
///
/// only used for handlebars templating
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, getset::Getters)]
pub struct VersionData {
    #[getset(get = "pub")]
    version: String,
    #[getset(get = "pub")]
    entries: Vec<Fragment>,
}

fn generate_template_data(
    release_files: impl Iterator<Item = Result<(Option<semver::Version>, Fragment), Error>>,
    suffix: Option<String>,
) -> Result<TemplateData, Error> {
    let versions = {
        use itertools::Itertools;
        let mut hm = HashMap::new();
        for r in release_files {
            let (version, fragment) = r?;

            if let Some(version) = version {
                hm.entry(version.to_string())
            } else {
                hm.entry("unreleased".to_string())
            }
            .or_insert_with(Vec::new)
            .push(fragment);
        }
        hm.into_iter()
            .map(|(version, entries)| VersionData { version, entries })
            .sorted_by(|va, vb| va.version.cmp(&vb.version))
    };

    Ok(TemplateData {
        versions: versions.collect(),
        suffix,
    })
}

#[cfg(test)]
mod tests {
    use crate::fragment::FragmentData;

    use super::*;
    use predicates::prelude::*;

    #[test]
    fn test_template_data_is_sorted() {
        let result = generate_template_data(
            [
                Ok((
                    Some(semver::Version::new(0, 2, 0)),
                    Fragment::new(
                        {
                            let mut hm = HashMap::new();
                            hm.insert("issue".to_string(), FragmentData::Int(123));
                            hm
                        },
                        "text of fragment for version 0.2.0".to_string(),
                    ),
                )),
                Ok((
                    Some(semver::Version::new(0, 1, 0)),
                    Fragment::new(
                        {
                            let mut hm = HashMap::new();
                            hm.insert("issue".to_string(), FragmentData::Int(345));
                            hm
                        },
                        "text of fragment for version 0.1.0".to_string(),
                    ),
                )),
            ]
            .into_iter(),
            None,
        );

        assert!(result.is_ok());
        let result = result.unwrap();

        let versions = result.versions;
        assert_eq!(versions[0].version, "0.1.0");
        assert_eq!(versions[1].version, "0.2.0");
    }

    #[test]
    fn default_template_renders_with_empty_data() {
        let hb = crate::template::new_handlebars(crate::consts::DEFAULT_TEMPLATE).unwrap();
        let data: HashMap<String, Vec<String>> = HashMap::new();
        let template = hb.render(crate::consts::INTERNAL_TEMPLATE_NAME, &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("CHANGELOG").eval(&template),
            "Does not contain 'CHANGELOG': {template}"
        );
    }

    #[test]
    fn default_template_renders_with_one_entry() {
        let hb = crate::template::new_handlebars(crate::consts::DEFAULT_TEMPLATE).unwrap();
        let mut data: HashMap<String, Vec<_>> = HashMap::new();
        data.insert(
            "versions".to_string(),
            vec![VersionData {
                version: "0.1.0".to_string(),
                entries: vec![Fragment::new(
                    {
                        let mut hdr = HashMap::new();
                        hdr.insert("issue".to_string(), FragmentData::Int(123));
                        hdr.insert("type".to_string(), FragmentData::Str("Bugfix".to_string()));
                        hdr
                    },
                    "test for 0.1.0".to_string(),
                )],
            }],
        );
        let template = hb.render(crate::consts::INTERNAL_TEMPLATE_NAME, &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("## v0.1.0").eval(&template),
            "Does not contain '## v0.1.0': {template}"
        );

        assert!(
            predicates::str::contains("test for 0.1.0").eval(&template),
            "Does not contain 'test text': {template}"
        );
    }

    #[test]
    fn default_template_renders_with_one_entry_with_header() {
        let hb = crate::template::new_handlebars(crate::consts::DEFAULT_TEMPLATE).unwrap();
        let mut data: HashMap<String, Vec<_>> = HashMap::new();
        data.insert(
            "versions".to_string(),
            vec![VersionData {
                version: "0.1.0".to_string(),
                entries: vec![Fragment::new(
                    {
                        let mut hdr = HashMap::new();
                        hdr.insert("issue".to_string(), FragmentData::Int(123));
                        hdr.insert("type".to_string(), FragmentData::Str("Bugfix".to_string()));
                        hdr
                    },
                    "test for 0.1.0".to_string(),
                )],
            }],
        );
        let template = hb.render(crate::consts::INTERNAL_TEMPLATE_NAME, &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("(#123)").eval(&template),
            "Does not contain '(#123)': {template}"
        );
    }

    #[test]
    fn default_template_renders_versions_sorted() {
        let hb = crate::template::new_handlebars(crate::consts::DEFAULT_TEMPLATE).unwrap();
        let mut data: HashMap<String, Vec<_>> = HashMap::new();
        data.insert(
            "versions".to_string(),
            vec![
                VersionData {
                    version: "0.1.0".to_string(),
                    entries: vec![Fragment::new(
                        {
                            let mut hdr = HashMap::new();
                            hdr.insert("issue".to_string(), FragmentData::Int(123));
                            hdr.insert("type".to_string(), FragmentData::Str("Bugfix".to_string()));
                            hdr
                        },
                        "test for 0.1.0".to_string(),
                    )],
                },
                VersionData {
                    version: "0.2.0".to_string(),
                    entries: vec![Fragment::new(
                        {
                            let mut hdr = HashMap::new();
                            hdr.insert("issue".to_string(), FragmentData::Int(234));
                            hdr.insert(
                                "type".to_string(),
                                FragmentData::Str("Feature".to_string()),
                            );
                            hdr
                        },
                        "test for 0.2.0".to_string(),
                    )],
                },
            ],
        );
        let template = hb.render(crate::consts::INTERNAL_TEMPLATE_NAME, &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("## v0.1.0").eval(&template),
            "Does not contain '## v0.1.0': {template}"
        );
        assert!(
            predicates::str::contains("## v0.2.0").eval(&template),
            "Does not contain '## v0.2.0': {template}"
        );

        let line_number_of_010 = {
            template
                .lines()
                .enumerate()
                .find(|(_n, line)| *line == "## v0.1.0")
                .map(|(n, _)| n)
                .unwrap()
        };

        let line_number_of_020 = {
            template
                .lines()
                .enumerate()
                .find(|(_n, line)| *line == "## v0.2.0")
                .map(|(n, _)| n)
                .unwrap()
        };

        assert!(
            line_number_of_020 < line_number_of_010,
            "line with v0.1.0 should come _after_ line with v0.2.0: {template}"
        );
    }
}
