use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;

use crate::config::Configuration;
use crate::error::Error;
use crate::format::Format;

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct NewCommand {
    interactive: bool,
    edit: bool,
    format: Format,
}

impl crate::command::Command for NewCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> miette::Result<()> {
        let unreleased_dir_path = ensure_fragment_dir(workdir, config)?;

        let new_file_path = {
            let new_file_name = format!(
                "{ts}.md",
                ts = {
                    time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Iso8601::DEFAULT)
                        .map_err(Error::from)
                        .into_diagnostic()?
                },
            );
            unreleased_dir_path.join(new_file_name)
        };

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&new_file_path)
            .map_err(Error::from)
            .into_diagnostic()?;

        let fragment = crate::fragment::Fragment::empty();

        let serialized_fragment = {
            serde_yaml::to_string(&fragment)
                .map_err(Error::from)
                .into_diagnostic()?
        };
        write!(file, "{}", serialized_fragment)
            .map_err(Error::from)
            .into_diagnostic()?;
        file.sync_all().map_err(Error::from).into_diagnostic()
    }
}

fn ensure_fragment_dir(workdir: &Path, config: &Configuration) -> miette::Result<PathBuf> {
    let unreleased_dir_path = workdir
        .join(config.fragment_dir())
        .join(crate::consts::UNRELEASED_DIR_NAME);
    std::fs::create_dir_all(&unreleased_dir_path)
        .map_err(Error::from)
        .into_diagnostic()?;

    Ok(unreleased_dir_path)
}
