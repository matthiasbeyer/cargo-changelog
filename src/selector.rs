use std::path::{Path, PathBuf};

use crate::{cli::Selector, error::Error};

pub struct SelectorExecutor<'sel> {
    selector: Option<&'sel Selector>,
}

impl<'sel> SelectorExecutor<'sel> {
    pub fn new(selector: Option<&'sel Selector>) -> Self {
        Self { selector }
    }

    pub fn run(
        &self,
        workdir: &Path,
        config: &crate::config::Configuration,
    ) -> Result<Vec<PathBuf>, Error> {
        match self.selector.as_ref() {
            None | Some(Selector::Unreleased) => {
                log::debug!("Showing unreleased");
                let unreleased_dir_path = workdir
                    .join(config.fragment_dir())
                    .join(crate::consts::UNRELEASED_DIR_NAME);

                Self::walk_dir(unreleased_dir_path)
                    .filter_map(Self::result_dir_entry_to_pathbuf)
                    .filter(|r| !Self::is_gitkeep(r))
                    .collect::<Result<Vec<PathBuf>, Error>>()
            }
            Some(Selector::Exact { exact }) => {
                log::debug!("Showing exact {exact}");
                let path = workdir.join(config.fragment_dir()).join(exact);
                if !path.exists() {
                    log::warn!("Version does not exist: {exact}");
                    return Ok(vec![]);
                }

                Self::walk_dir(path)
                    .filter_map(Self::result_dir_entry_to_pathbuf)
                    .filter(|r| !Self::is_gitkeep(r))
                    .collect::<Result<Vec<PathBuf>, Error>>()
            }
            Some(Selector::Range { from, until }) => {
                log::debug!("Showing range from {from} until {until}");
                let from = semver::Version::parse(from)?;
                let until = semver::Version::parse(until)?;

                let fragment_dir_path = workdir.join(config.fragment_dir());

                Self::walk_dir(fragment_dir_path)
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
                    .filter_map(Self::result_dir_entry_to_pathbuf)
                    .filter(|r| !Self::is_gitkeep(r))
                    .collect::<Result<Vec<PathBuf>, Error>>()
            }
        }
    }

    fn walk_dir(path: PathBuf) -> walkdir::IntoIter {
        walkdir::WalkDir::new(path)
            .follow_links(false)
            .max_open(100)
            .same_file_system(true)
            .into_iter()
    }

    fn result_dir_entry_to_pathbuf(
        rde: Result<walkdir::DirEntry, walkdir::Error>,
    ) -> std::option::Option<std::result::Result<std::path::PathBuf, Error>> {
        match rde {
            Ok(de) => de.path().is_file().then(|| de.path().to_path_buf()).map(Ok),
            Err(e) => Some(Err(Error::from(e))),
        }
    }

    fn is_gitkeep(rpath: &Result<PathBuf, Error>) -> bool {
        match rpath {
            Ok(path) => path.ends_with(".gitkeep"),
            Err(_) => true,
        }
    }
}
