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
        let (oks, errors): (Vec<_>, Vec<VerificationError>) =
            walkdir::WalkDir::new(workdir.join(config.fragment_dir()))
                .follow_links(false)
                .max_open(100)
                .same_file_system(true)
                .into_iter()
                .map(|rde| {
                    rde.map_err(VerificationError::from)
                        .and_then(|de| verify_entry(de.path()))
                })
                .partition_result();

        if !errors.is_empty() {
            return Err(Error::Verification(errors));
        }

        let mut iter = oks.into_iter();
        if let Some(first) = iter.next() {
            let (_oks, errors): (Vec<_>, Vec<_>) = iter
                .map(|elem| headers_equal(&first, elem))
                .partition_result();

            if !errors.is_empty() {
                return Err(Error::Verification(errors));
            }
        }

        Ok(())
    }
}

fn verify_entry(entry: &std::path::Path) -> Result<Fragment, VerificationError> {
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
}

fn headers_equal(left: &Fragment, right: Fragment) -> Result<(), VerificationError> {
    left.header()
        .keys()
        .map(|key| {
            right
                .header()
                .keys()
                .contains(key)
                .then_some(())
                .ok_or_else(|| VerificationError::HeaderLayout {
                    right_keys: right.header().keys().cloned().collect(),
                    left_keys: left.header().keys().cloned().collect(),
                })
        })
        .collect()
}
