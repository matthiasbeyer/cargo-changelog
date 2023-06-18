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
        let (_oks, errors): (Vec<_>, Vec<Error>) =
            walkdir::WalkDir::new(workdir.join(config.fragment_dir()))
                .follow_links(false)
                .max_open(100)
                .same_file_system(true)
                .into_iter()
                .filter_map(|rde| match rde {
                    Ok(de) => de.path().is_file().then(|| de.path().to_path_buf()).map(Ok),
                    Err(e) => Some(Err(e)),
                })
                .map(|rde| -> Result<_, Error> {
                    let de = rde.map_err(VerificationError::from)?;
                    let path = de.as_path();

                    if crate::command::common::get_version_from_path(path)
                        .map_err(VerificationError::from)?
                        .is_none()
                    {
                        log::warn!("No version: {}", path.display());
                    }

                    std::fs::OpenOptions::new()
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
                        .map_err(|ve| Error::Verification(ve))
                })
                .partition_result();

        if !errors.is_empty() {
            return Err(Error::Multiple { errors });
        }

        Ok(())
    }
}
