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
        let (_oks, errors): (Vec<_>, Vec<VerificationError>) =
            walkdir::WalkDir::new(workdir.join(config.fragment_dir()))
                .follow_links(false)
                .max_open(100)
                .same_file_system(true)
                .into_iter()
                .map(|rde| -> Result<_, VerificationError> {
                    let de = rde.map_err(VerificationError::from)?;
                    let path = de.path();

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
                })
                .partition_result();

        if !errors.is_empty() {
            return Err(Error::Verification(errors));
        }

        Ok(())
    }
}
