use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use dialoguer::Confirm;
use dialoguer::Input;

use crate::cli::TextProvider;
use crate::cli::KV;
use crate::config::Configuration;
use crate::config::GitSetting;
use crate::error::Error;
use crate::error::FragmentError;
use crate::error::InteractiveError;
use crate::format::Format;
use crate::fragment::Crawler;
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
    git: Option<GitSetting>,
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
                let cli_set: Option<FragmentData> = match self
                    .set
                    .iter()
                    .find(|kv| kv.key() == key)
                    .map(KV::value)
                    .map(|val| FragmentData::parse(val))
                {
                    Some(Ok(val)) => Some(val),
                    Some(Err(e)) => return Some(Err(e)),
                    None => None,
                };
                let crawler = data_desc.crawler();
                let default_value = data_desc.default_value();

                // if there is a default value, but its type is not correct, fail
                if let Some(default) = default_value.as_ref() {
                    if !data_desc.fragment_type().matches(&default) {
                        return Some(Err(FragmentError::DataType {
                            exp: data_desc.fragment_type().type_name().to_string(),
                            recv: default.type_name().to_string(),
                            field_name: key.to_string(),
                        }));
                    }
                }

                // if there is a CLI provided value, but its type is not correct, fail
                if let Some(clival) = cli_set.as_ref() {
                    if !data_desc.fragment_type().matches(clival) {
                        return Some(Err(FragmentError::DataType {
                            exp: data_desc.fragment_type().type_name().to_string(),
                            recv: clival.type_name().to_string(),
                            field_name: key.to_string(),
                        }));
                    }
                }

                match (default_value, cli_set, crawler) {
                    (Some(default), None, None) => {
                        if self.interactive {
                            interactive_edit(key, default, data_desc)
                                .map_err(FragmentError::from)
                                .transpose()
                        } else {
                            Some(Ok((key.to_string(), default.clone())))
                        }
                    }

                    (_, Some(clival), _) => {
                        if self.interactive {
                            interactive_edit(key, &clival, data_desc)
                                .map_err(FragmentError::from)
                                .transpose()
                        } else {
                            Some(Ok((key.to_string(), clival.clone())))
                        }
                    }

                    (_, _, Some(crawler)) => {
                        let crawled_value = match crawl_with_crawler(
                            crawler,
                            key,
                            workdir,
                            data_desc.fragment_type(),
                        ) {
                            Err(e) => return Some(Err(e)),
                            Ok(val) => val,
                        };

                        if !data_desc.fragment_type().matches(&crawled_value) {
                            return Some(Err(FragmentError::DataType {
                                exp: data_desc.fragment_type().type_name().to_string(),
                                recv: crawled_value.type_name().to_string(),
                                field_name: key.to_string(),
                            }));
                        }

                        Some(Ok((key.to_string(), crawled_value)))
                    }

                    (None, None, None) => {
                        if data_desc.required() {
                            Some(Err(FragmentError::RequiredValueMissing(key.to_string())))
                        } else {
                            None
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

        match self.git.as_ref().or_else(|| config.git().as_ref()) {
            Some(GitSetting::Add) => {
                // We use the simple approach here and use std::command::Command for calling git
                Command::new("git")
                    .arg("add")
                    .arg(&new_file_path)
                    .stderr(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .output()?;
            }
            Some(GitSetting::Commit) => {
                Command::new("git")
                    .arg("add")
                    .arg(&new_file_path)
                    .stderr(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .output()?;

                let mut commit_cmd = Command::new("git");
                commit_cmd.arg("commit").arg(&new_file_path);

                if let Some(message) = config.git_commit_message().as_ref() {
                    commit_cmd.arg("--message").arg(message);
                }

                if config.git_commit_signoff() {
                    commit_cmd.arg("--signoff");
                }

                commit_cmd
                    .stderr(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .output()?;
            }
            None => {}
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
    }
}

fn crawl_with_crawler(
    crawler: &Crawler,
    field_name: &str,
    workdir: &Path,
    expected_type: &FragmentDataType,
) -> Result<FragmentData, FragmentError> {
    let (command_str, mut command) = match crawler {
        Crawler::Path(path) => (path.display().to_string(), Command::new(workdir.join(path))),
        Crawler::Command(s) => {
            let mut cmd = comma::parse_command(s)
                .ok_or_else(|| FragmentError::NoValidCommand(s.to_string()))?;
            let binary = cmd.remove(0);
            let mut command = Command::new(binary);
            command.args(cmd);
            (s.to_string(), command)
        }
    };

    let std::process::Output { status, stdout, .. } = command
        .stderr(std::process::Stdio::inherit())
        .env("CARGO_CHANGELOG_CRAWLER_FIELD_NAME", field_name.to_string())
        .env(
            "CARGO_CHANGELOG_CRAWLER_FIELD_TYPE",
            expected_type.type_name().to_string(),
        )
        .output()
        .map_err(FragmentError::from)?;

    if status.success() {
        log::info!("Executed crawl successfully");
        let out = String::from_utf8(stdout)
            .map_err(|e| FragmentError::NoUtf8Output(command_str, e))?
            .trim()
            .to_string();
        log::info!("crawled = '{}'", out);
        let data = FragmentData::parse(&out)?;
        if expected_type.matches(&data) {
            Ok(data)
        } else {
            Err(FragmentError::DataType {
                exp: expected_type.type_name(),
                recv: data.type_name().to_string(),
                field_name: field_name.to_string(),
            })
        }
    } else {
        Err(FragmentError::from(FragmentError::CommandNoSuccess(
            command_str,
        )))
    }
}
