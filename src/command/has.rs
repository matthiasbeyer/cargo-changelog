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
            HasFormat::Json => todo!(),
        }
    }
}
