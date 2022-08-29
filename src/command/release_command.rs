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

        let template_data = {
            #[derive(Debug, serde::Serialize)]
            struct VersionData {
                version: String,
                entries: Vec<Fragment>,
            }
            let versions = {
                use itertools::Itertools;
                let mut hm = HashMap::new();
                for r in load_release_files(workdir, config) {
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
            hm
        };

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
