use handlebars::Handlebars;
use miette::IntoDiagnostic;

use crate::error::Error;

mod reverse_helper;
mod sort_versions_helper;

pub fn new_handlebars(template_source: &str) -> miette::Result<Handlebars> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(crate::consts::INTERNAL_TEMPLATE_NAME, template_source)
        .map_err(Error::from)
        .into_diagnostic()?;
    handlebars.register_helper(
        "sort_versions",
        Box::new(self::sort_versions_helper::sort_versions),
    );
    handlebars.register_helper("reverse", Box::new(self::reverse_helper::ReverseHelper));
    Ok(handlebars)
}
