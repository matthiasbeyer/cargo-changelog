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
                .map(|rde| {
                    rde.map(|de| verify_entry(de.path()))
                        .map_err(VerificationError::from)
                })
                .partition_result();

        if !errors.is_empty() {
            Err(Error::Verification(errors))
        } else {
            Ok(())
        }
    }
}

fn verify_entry(entry: &std::path::Path) -> Result<(), VerificationError> {
    if crate::command::common::get_version_from_path(entry)?.is_none() {
        log::warn!("No version: {}", entry.display());
    }

    std::fs::OpenOptions::new()
        .read(true)
        .create(false)
        .write(false)
        .open(entry)
        .map_err(FragmentError::from)
        .map(BufReader::new)
        .and_then(|mut reader| Fragment::from_reader(&mut reader))
        .map_err(|e| VerificationError::FragmentParsing(entry.to_path_buf(), e))
        .map(|_| ())
}
