use std::io::Write;
use std::{collections::HashMap, io::BufReader, path::Path};

use handlebars::Handlebars;
use miette::IntoDiagnostic;

use crate::{config::Configuration, error::Error, fragment::Fragment};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct ReleaseCommand {}

impl crate::command::Command for ReleaseCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> miette::Result<()> {
        let template = {
            let template_path = workdir
                .join(config.fragment_dir())
                .join(config.template_path());
            let template_source = std::fs::read_to_string(template_path)
                .map_err(Error::from)
                .into_diagnostic()?;

            let mut handlebars = Handlebars::new();
            handlebars
                .register_template_string(crate::consts::INTERNAL_TEMPLATE_NAME, template_source)
                .map_err(Error::from)
                .into_diagnostic()?;
            handlebars
        };

        let template_data = compute_template_data(load_release_files(workdir, config))?;

        let changelog_contents = template
            .render(crate::consts::INTERNAL_TEMPLATE_NAME, &template_data)
            .map_err(Error::from)
            .into_diagnostic()?;

        let mut changelog_file = std::fs::OpenOptions::new()
            .create(true)
            .append(false)
            .truncate(true)
            .open(workdir.join(config.template_path()))
            .map_err(Error::from)
            .into_diagnostic()?;

        write!(changelog_file, "{}", changelog_contents)
            .map_err(Error::from)
            .into_diagnostic()?;
        changelog_file
            .sync_all()
            .map_err(Error::from)
            .into_diagnostic()
    }
}

fn load_release_files(
    workdir: &Path,
    config: &Configuration,
) -> impl Iterator<Item = miette::Result<(semver::Version, Fragment)>> {
    walkdir::WalkDir::new(workdir.join(config.fragment_dir()))
        .follow_links(false)
        .max_open(100)
        .same_file_system(true)
        .into_iter()
        .filter_map(|rde| match rde {
            Err(e) => Some(Err(e)),
            Ok(de) => {
                if de.file_type().is_file() {
                    Some(Ok(de))
                } else {
                    None
                }
            }
        })
        .map(|rde| {
            rde.map_err(Error::from).into_diagnostic().and_then(|de| {
                let version: semver::Version = de
                    .path()
                    .components()
                    .find_map(|comp| match comp {
                        std::path::Component::Normal(comp) => {
                            let s = comp
                                .to_str()
                                .ok_or_else(|| miette::miette!("UTF8 Error in path: {:?}", comp));
                            Some(s.and_then(|s| semver::Version::parse(s).into_diagnostic()))
                        }
                        _ => None,
                    })
                    .ok_or_else(|| {
                        miette::miette!("Did not find version for path: {}", de.path().display())
                    })??;

                let fragment = std::fs::OpenOptions::new()
                    .read(true)
                    .create(false)
                    .write(false)
                    .open(de.path())
                    .map_err(Error::from)
                    .into_diagnostic()
                    .map(BufReader::new)
                    .and_then(|mut reader| Fragment::from_reader(&mut reader))?;

                Ok((version, fragment))
            })
        })
}

/// Helper type for storing version associated with Fragments
///
/// only used for handlebars templating
#[derive(Debug, serde::Serialize)]
struct VersionData {
    version: String,
    entries: Vec<Fragment>,
}

fn compute_template_data(
    release_files: impl Iterator<Item = miette::Result<(semver::Version, Fragment)>>,
) -> miette::Result<HashMap<String, Vec<VersionData>>> {
    let versions = {
        use itertools::Itertools;
        let mut hm = HashMap::new();
        for r in release_files {
            let (version, fragment) = r?;
            hm.entry(version.to_string())
                .or_insert_with(Vec::new)
                .push(fragment);
        }
        hm.into_iter()
            .map(|(version, entries)| VersionData { version, entries })
            .sorted_by(|va, vb| va.version.cmp(&vb.version))
    };

    let mut hm: HashMap<String, Vec<VersionData>> = HashMap::new();
    hm.insert("versions".to_string(), versions.collect());
    Ok(hm)
}

#[cfg(test)]
mod tests {
    use crate::fragment::FragmentData;

    use super::*;
    use predicates::prelude::*;

    #[test]
    fn test_template_data_is_sorted() {
        let result = compute_template_data(
            [
                Ok((
                    semver::Version::new(0, 2, 0),
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
                    semver::Version::new(0, 1, 0),
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
        );

        assert!(result.is_ok());
        let result = result.unwrap();

        let versions = result.get("versions").unwrap();
        assert_eq!(versions[0].version, "0.1.0");
        assert_eq!(versions[1].version, "0.2.0");
    }

    #[test]
    fn default_template_renders_with_empty_data() {
        let mut hb = Handlebars::new();
        let data: HashMap<String, Vec<String>> = HashMap::new();
        hb.register_template_string("t", crate::consts::DEFAULT_TEMPLATE)
            .unwrap();
        let template = hb.render("t", &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("CHANGELOG").eval(&template),
            "Does not contain 'CHANGELOG': {}",
            template
        );
    }

    #[test]
    fn default_template_renders_with_one_entry() {
        let mut hb = Handlebars::new();
        let mut data: HashMap<String, Vec<_>> = HashMap::new();
        data.insert(
            "versions".to_string(),
            vec![VersionData {
                version: "0.1.0".to_string(),
                entries: vec![Fragment::new(
                    {
                        let mut hdr = HashMap::new();
                        hdr.insert("issue".to_string(), FragmentData::Int(123));
                        hdr
                    },
                    "test for 0.1.0".to_string(),
                )],
            }],
        );
        hb.register_template_string("t", crate::consts::DEFAULT_TEMPLATE)
            .unwrap();
        let template = hb.render("t", &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("## v0.1.0").eval(&template),
            "Does not contain '## v0.1.0': {}",
            template
        );

        assert!(
            predicates::str::contains("test for 0.1.0").eval(&template),
            "Does not contain 'test text': {}",
            template
        );
    }

    #[test]
    fn default_template_renders_with_one_entry_with_header() {
        let mut hb = Handlebars::new();
        let mut data: HashMap<String, Vec<_>> = HashMap::new();
        data.insert(
            "versions".to_string(),
            vec![VersionData {
                version: "0.1.0".to_string(),
                entries: vec![Fragment::new(
                    {
                        let mut hdr = HashMap::new();
                        hdr.insert("issue".to_string(), FragmentData::Int(123));
                        hdr
                    },
                    "test for 0.1.0".to_string(),
                )],
            }],
        );
        hb.register_template_string("t", crate::consts::DEFAULT_TEMPLATE)
            .unwrap();
        let template = hb.render("t", &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("(#123)").eval(&template),
            "Does not contain '(#123)': {}",
            template
        );
    }

    #[test]
    fn default_template_renders_versions_sorted() {
        let mut hb = Handlebars::new();
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
                            hdr
                        },
                        "test for 0.2.0".to_string(),
                    )],
                },
            ],
        );
        hb.register_template_string("t", crate::consts::DEFAULT_TEMPLATE)
            .unwrap();
        let template = hb.render("t", &data);
        assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
        let template = template.unwrap();

        assert!(
            predicates::str::contains("## v0.1.0").eval(&template),
            "Does not contain '## v0.1.0': {}",
            template
        );
        assert!(
            predicates::str::contains("## v0.2.0").eval(&template),
            "Does not contain '## v0.2.0': {}",
            template
        );

        let line_number_of_010 = {
            template
                .lines()
                .enumerate()
                .filter(|(_n, line)| *line == "## v0.1.0")
                .next()
                .map(|(n, _)| n)
                .unwrap()
        };

        let line_number_of_020 = {
            template
                .lines()
                .enumerate()
                .filter(|(_n, line)| *line == "## v0.2.0")
                .next()
                .map(|(n, _)| n)
                .unwrap()
        };

        assert!(
            line_number_of_020 < line_number_of_010,
            "line with v0.1.0 should come _after_ line with v0.2.0: {}",
            template
        );
    }
}
