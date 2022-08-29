use handlebars::Handlebars;
use miette::IntoDiagnostic;

use crate::error::Error;

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct ReleaseCommand {}

impl crate::command::Command for ReleaseCommand {
    fn execute(
        self,
        workdir: &std::path::Path,
        config: &crate::config::Configuration,
    ) -> miette::Result<()> {
        let template = {
            let template_path = workdir.join(config.fragment_dir()).join(config.template_path());
            let template_source = std::fs::read_to_string(template_path).map_err(Error::from).into_diagnostic()?;

            let mut handlebars = Handlebars::new();
            handlebars.register_template_string(crate::consts::INTERNAL_TEMPLATE_NAME, template_source)
                .map_err(Error::from)
                .into_diagnostic()?;
        };
        todo!()
    }
}
