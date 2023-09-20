use std::{
    collections::HashMap,
    io::BufReader,
    io::Write,
    path::{Path, PathBuf},
};

use is_terminal::IsTerminal;
use yansi::Paint;

use crate::{
    cli::{ShowFormat, ShowRange},
    config::Configuration,
    error::{Error, FragmentError},
    fragment::Fragment,
};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct Show {
    format: Option<crate::cli::ShowFormat>,
    range: Option<ShowRange>,
}

impl crate::command::Command for Show {
    fn execute(self, workdir: &Path, config: &Configuration) -> Result<(), Error> {
        let walk_dir = |path| {
            walkdir::WalkDir::new(path)
                .follow_links(false)
                .max_open(100)
                .same_file_system(true)
                .into_iter()
        };

        let result_dir_entry_to_pathbuf = |rde: Result<walkdir::DirEntry, _>| match rde {
            Ok(de) => de.path().is_file().then(|| de.path().to_path_buf()).map(Ok),
            Err(e) => Some(Err(Error::from(e))),
        };

        let is_gitkeep = |rpath: &Result<PathBuf, _>| match rpath {
            Ok(path) => path.ends_with(".gitkeep"),
            Err(_) => true,
        };

        let pathes = match self.range {
            None | Some(ShowRange::Unreleased) => {
                log::debug!("Showing unreleased");
                let unreleased_dir_path = workdir
                    .join(config.fragment_dir())
                    .join(crate::consts::UNRELEASED_DIR_NAME);
                walk_dir(unreleased_dir_path)
                    .filter_map(result_dir_entry_to_pathbuf)
                    .filter(|r| !is_gitkeep(r))
                    .collect::<Result<Vec<PathBuf>, Error>>()?
            }
            Some(ShowRange::Exact { exact }) => {
                log::debug!("Showing exact {exact}");
                let path = workdir.join(config.fragment_dir()).join(&exact);
                if !path.exists() {
                    return Err(Error::ExactVersionDoesNotExist { version: exact });
                }
                walk_dir(path)
                    .filter_map(result_dir_entry_to_pathbuf)
                    .filter(|r| !is_gitkeep(r))
                    .collect::<Result<Vec<PathBuf>, Error>>()?
            }
            Some(ShowRange::Range { from, until }) => {
                log::debug!("Showing range from {from} until {until}");
                let from = semver::Version::parse(&from)?;
                let until = semver::Version::parse(&until)?;

                let fragment_dir_path = workdir.join(config.fragment_dir());
                walk_dir(fragment_dir_path)
                    .filter_entry(|de| {
                        log::debug!("Looking at {de:?}");
                        if de.path().is_dir() {
                            true
                        } else if de.path().is_file() {
                            de.path().components().any(|comp| match comp {
                                std::path::Component::Normal(osstr) => osstr
                                    .to_str()
                                    .map(|s| {
                                        if let Ok(version) = semver::Version::parse(s) {
                                            version > from && version < until
                                        } else {
                                            false
                                        }
                                    })
                                    .unwrap_or(false),
                                _ => false,
                            })
                        } else {
                            false
                        }
                    })
                    .filter_map(result_dir_entry_to_pathbuf)
                    .filter(|r| !is_gitkeep(r))
                    .collect::<Result<Vec<PathBuf>, Error>>()?
            }
        };

        log::trace!("Looking at: {pathes:?}");
        let fragments = pathes.into_iter().map(|path| {
            std::fs::OpenOptions::new()
                .read(true)
                .create(false)
                .write(false)
                .open(&path)
                .map_err(FragmentError::from)
                .map(BufReader::new)
                .and_then(|mut reader| {
                    Fragment::from_reader(&mut reader).map(|f| (path.to_path_buf(), f))
                })
                .map_err(|e| Error::FragmentError(e, path.to_path_buf()))
        });

        match self.format {
            None | Some(ShowFormat::Text) => pretty_print(fragments),
            Some(ShowFormat::Json) => json_print(fragments),
        }
    }
}

fn pretty_print(
    mut iter: impl Iterator<Item = Result<(PathBuf, Fragment), Error>>,
) -> Result<(), Error> {
    let out = std::io::stdout();
    let mut output = out.lock();

    let is_terminal = std::io::stdout().is_terminal();
    if !is_terminal {
        yansi::Paint::disable()
    }

    iter.try_for_each(|fragment| {
        let (path, fragment) = fragment?;
        writeln!(output, "{}", Paint::new(path.display()).bold())?;
        fragment.header().iter().try_for_each(|(key, value)| {
            writeln!(
                output,
                "{key}: {value}",
                key = Paint::new(key).italic(),
                value = value.display()
            )?;
            Ok(()) as Result<(), Error>
        })?;

        writeln!(output, "{text}", text = fragment.text())?;
        writeln!(output)?;
        Ok(())
    })
}

fn json_print(iter: impl Iterator<Item = Result<(PathBuf, Fragment), Error>>) -> Result<(), Error> {
    let v = iter.collect::<Result<HashMap<PathBuf, Fragment>, _>>()?;
    let out = std::io::stdout();
    let output = out.lock();
    serde_json::to_writer(output, &v).map_err(Error::from)
}
