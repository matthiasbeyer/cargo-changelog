use crate::cli::{HasFormat, Selector};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct HasCommand {
    format: Option<HasFormat>,
    selector: Selector,
}

impl crate::command::Command for HasCommand {
    fn execute(
        self,
        workdir: &std::path::Path,
        config: &crate::config::Configuration,
    ) -> Result<Option<std::process::ExitCode>, crate::error::Error> {
        let pathes =
            crate::selector::SelectorExecutor::new(Some(&self.selector)).run(workdir, config)?;

        match self.format.unwrap_or_default() {
            HasFormat::ExitCode => {
                if pathes.is_empty() {
                    Ok(Some(std::process::ExitCode::FAILURE))
                } else {
                    Ok(Some(std::process::ExitCode::SUCCESS))
                }
            }
            HasFormat::Json => {
                let reply = HasReply {
                    cargo_changelog: CargoChangelogMetadata::default(),
                    selector: self.selector.clone(),
                    pathes,
                };

                let reply = serde_json::to_string(&reply)?;

                println!("{reply}");
                Ok(None)
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct HasReply {
    #[serde(rename = "cargo-changelog")]
    cargo_changelog: CargoChangelogMetadata,
    selector: Selector,
    pathes: Vec<std::path::PathBuf>,
}

#[derive(Debug, serde::Serialize)]
struct CargoChangelogMetadata {
    version: String,
}

impl Default for CargoChangelogMetadata {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
