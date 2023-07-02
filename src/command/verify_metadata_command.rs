use std::{io::BufReader, path::Path};

use itertools::Itertools;

use crate::{
    config::Configuration,
    error::{Error, FragmentError, VerificationError},
    fragment::Fragment,
};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct VerifyMetadataCommand {}

impl crate::command::Command for VerifyMetadataCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> Result<(), Error> {
        let walkdir = workdir.join(config.fragment_dir());
        log::debug!("Walking {}", walkdir.display());
        let (_oks, errors): (Vec<_>, Vec<Error>) = walkdir::WalkDir::new(walkdir)
            .follow_links(false)
            .max_open(100)
            .same_file_system(true)
            .into_iter()
            .filter_map(|rde| match rde {
                Ok(de) => de.path().is_file().then(|| de.path().to_path_buf()).map(Ok),
                Err(e) => Some(Err(e)),
            })
            .filter_map(|rde| {
                let de = match rde {
                    Err(e) => return Some(Err(Error::from(VerificationError::from(e)))),
                    Ok(de) => de,
                };
                let path = de.as_path();
                log::trace!("Looking at {}", path.display());

                match crate::command::common::get_version_from_path(path)
                    .map_err(VerificationError::from)
                {
                    Err(e) => return Some(Err(Error::from(e))),
                    Ok(None) => return None,
                    Ok(_some) => {
                        // nothing
                    }
                }

                let res = std::fs::OpenOptions::new()
                    .read(true)
                    .create(false)
                    .write(false)
                    .open(path)
                    .map_err(FragmentError::from)
                    .map_err(|e| VerificationError::FragmentParsing(path.to_path_buf(), e))
                    .map(BufReader::new)
                    .and_then(|mut reader| {
                        let fragment = Fragment::from_reader(&mut reader).map_err(|e| {
                            VerificationError::FragmentParsing(path.to_path_buf(), e)
                        })?;

                        fragment
                            .header_matches_config(config.header_fields())
                            .map(|_| fragment)
                            .map_err(|errors| VerificationError::Multiple {
                                fragment_path: path.to_path_buf(),
                                multiple: errors,
                            })
                    })
                    .map_err(Error::Verification);

                Some(res)
            })
            .partition_result();

        if !errors.is_empty() {
            return Err(Error::Multiple { errors });
        }

        Ok(())
    }
}
