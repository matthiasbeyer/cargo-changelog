use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use dialoguer::Confirm;
use dialoguer::Input;

use crate::cli::TextProvider;
use crate::cli::KV;
use crate::config::Configuration;
use crate::error::Error;
use crate::error::FragmentError;
use crate::error::InteractiveError;
use crate::format::Format;
use crate::fragment::FragmentData;
use crate::fragment::FragmentDataDesc;
use crate::fragment::FragmentDataType;

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct NewCommand {
    interactive: bool,
    edit: bool,
    format: Format,
    set: Vec<KV>,
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

        // Fill the fragment header with data
        *fragment.header_mut() = config
            .header_fields()
            .into_iter()
            .filter_map(|(key, data_desc)| {
                if let Some(default) = data_desc.default_value() {
                    if data_desc.fragment_type().matches(&default) {
                        if self.interactive {
                            interactive_edit(key, default, data_desc)
                                .map_err(FragmentError::from)
                                .transpose()
                        } else {
                            Some(Ok((key.to_string(), default.clone())))
                        }
                    } else {
                        Some(Err(FragmentError::DataType {
                            exp: data_desc.fragment_type().type_name().to_string(),
                            recv: default.type_name().to_string(),
                        }))
                    }
                } else {
                    if self.interactive {
                        interactive_provide(key, data_desc)
                            .map_err(FragmentError::from)
                            .transpose()
                    } else {
                        if data_desc.required() {
                            Some(Err(FragmentError::RequiredValueNotInteractive(
                                key.to_string(),
                            )))
                        } else {
                            self.set.iter().find(|kv| kv.key() == key).map(|kv| {
                                FragmentData::parse(kv.value()).map(|data| (key.to_string(), data))
                            })
                        }
                    }
                }
            })
            .collect::<Result<HashMap<String, FragmentData>, _>>()
            .map_err(|e| Error::FragmentError(e, new_file_path.to_path_buf()))?;

        fragment
            .write_to(&mut file, self.format)
            .map_err(|e| Error::FragmentError(e, new_file_path.to_path_buf()))?;
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

/// Ask interactively whether these values are okay or should be changed
fn interactive_edit(
    key: &str,
    value: &FragmentData,
    value_desc: &FragmentDataDesc,
) -> Result<Option<(String, FragmentData)>, InteractiveError> {
    let prompt = format!("Edit '{key}' = '{data}' ({type})?",
        key = key,
        data = value.display(),
        type = value.type_name());

    let confirmed = dialoguer::Confirm::new()
        .default(true)
        .show_default(true)
        .with_prompt(prompt)
        .interact_opt()
        .map_err(InteractiveError::from)?;

    match confirmed {
        None => Err(InteractiveError::Interrupted),
        Some(true) => Ok(Some((key.to_string(), value.clone()))),
        Some(false) => interactive_provide(key, value_desc),
    }
}

/// Let the user provide a value matching the description interactively
fn interactive_provide(
    key: &str,
    desc: &FragmentDataDesc,
) -> Result<Option<(String, FragmentData)>, InteractiveError> {
    match desc.fragment_type() {
        FragmentDataType::Bool => {
            let mut dialoguer = Confirm::new();
            dialoguer.with_prompt(format!("'{}'?", key));
            if let Some(data) = desc.default_value() {
                if let FragmentData::Bool(b) = data {
                    dialoguer.default(*b);
                } else {
                    return Err(InteractiveError::TypeError(
                        desc.fragment_type().clone(),
                        data.clone(),
                    ));
                }
            }

            let value = if desc.required() {
                dialoguer.interact().map_err(InteractiveError::from)?
            } else {
                let value = dialoguer.interact_opt().map_err(InteractiveError::from)?;
                match value {
                    None => return Ok(None),
                    Some(val) => val,
                }
            };

            Ok(Some((key.to_string(), FragmentData::Bool(value))))
        }
        FragmentDataType::Int => {
            let mut dialoguer = Input::<u64>::new();
            dialoguer.with_prompt(format!("Enter a number for '{}'", key));

            if let Some(data) = desc.default_value() {
                if let FragmentData::Int(i) = data {
                    dialoguer.default(*i);
                } else {
                    return Err(InteractiveError::TypeError(
                        desc.fragment_type().clone(),
                        data.clone(),
                    ));
                }
            }

            let value = dialoguer.interact_text().map_err(InteractiveError::from)?;
            Ok(Some((key.to_string(), FragmentData::Int(value))))
        }
        FragmentDataType::Str => {
            let mut dialoguer = Input::<String>::new();
            dialoguer.with_prompt(format!("Enter a text for '{}'", key));

            if let Some(data) = desc.default_value() {
                if let FragmentData::Str(s) = data {
                    dialoguer.default(s.to_string());
                } else {
                    return Err(InteractiveError::TypeError(
                        desc.fragment_type().clone(),
                        data.clone(),
                    ));
                }
            }

            let value = dialoguer.interact_text().map_err(InteractiveError::from)?;
            Ok(Some((key.to_string(), FragmentData::Str(value))))
        }
        FragmentDataType::List(_) => todo!(),
        FragmentDataType::Map(_) => todo!(),
    }
}
