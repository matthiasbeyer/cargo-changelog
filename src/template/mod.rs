use handlebars::Handlebars;
use miette::IntoDiagnostic;

use crate::error::Error;

mod reverse_helper;
mod sort_versions_helper;

pub mod helpers {
    pub use super::reverse_helper::ReverseHelper;
    pub use super::sort_versions_helper::sort_versions;
}

pub fn new_handlebars(template_source: &str) -> miette::Result<Handlebars> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(crate::consts::INTERNAL_TEMPLATE_NAME, template_source)
        .map_err(Error::from)
        .into_diagnostic()?;
    handlebars.register_helper(
        "sort_versions",
        Box::new(crate::template::helpers::sort_versions),
    );
    handlebars.register_helper("reverse", Box::new(crate::template::helpers::ReverseHelper));
    Ok(handlebars)
}
