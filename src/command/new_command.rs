use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use miette::IntoDiagnostic;

use crate::cli::TextProvider;
use crate::config::Configuration;
use crate::error::Error;
use crate::format::Format;

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct NewCommand {
    edit: bool,
    format: Format,
    text: Option<TextProvider>,
}

impl crate::command::Command for NewCommand {
    fn execute(self, workdir: &Path, config: &Configuration) -> Result<(), Error> {
        let unreleased_dir_path = ensure_fragment_dir(workdir, config)?;

        let new_file_path = {
            let new_file_name = format!(
                "{ts}.md",
                ts = {
                    time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Iso8601::DEFAULT)?
                },
            );
            unreleased_dir_path.join(new_file_name)
        };

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&new_file_path)?;

        let mut fragment = crate::fragment::Fragment::empty();

        if let Some(text_provider) = self.text.as_ref() {
            let text = text_provider.read()?;
            fragment.set_text(text);
        }

        fragment.fill_header_from(config.header_fields())?;

        fragment.write_to(&mut file, self.format)?;
        file.sync_all()?;
        drop(file);

        if self.edit {
            let mut editor_command = get_editor_command()?;
            let std::process::Output { status, .. } = editor_command
                .arg(&new_file_path)
                .stderr(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .output()?;

            if status.success() {
                log::info!("Successfully edited");
            } else {
                log::error!("Failure editing {}", new_file_path.display());
            }
        }

        Ok(())
    }
}

fn ensure_fragment_dir(workdir: &Path, config: &Configuration) -> Result<PathBuf, Error> {
    let unreleased_dir_path = workdir
        .join(config.fragment_dir())
        .join(crate::consts::UNRELEASED_DIR_NAME);
    std::fs::create_dir_all(&unreleased_dir_path)?;
    Ok(unreleased_dir_path)
}

fn get_editor_command() -> Result<Command, Error> {
    let editor = match std::env::var("EDITOR") {
        Ok(editor) => editor,
        Err(std::env::VarError::NotPresent) => match std::env::var("VISUAL") {
            Ok(editor) => editor,
            Err(std::env::VarError::NotPresent) => return Err(Error::EditorEnvNotSet),
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(Error::EnvNotUnicode("VISUAL".to_string()))
            }
        },
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(Error::EnvNotUnicode("EDITOR".to_string()))
        }
    };

    Ok(Command::new(editor))
}
