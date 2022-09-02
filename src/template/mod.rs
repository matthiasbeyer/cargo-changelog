use handlebars::Handlebars;

use crate::error::Error;

mod reverse_helper;
mod sort_versions_helper;

pub fn new_handlebars(template_source: &str) -> Result<Handlebars, Error> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_template_string(crate::consts::INTERNAL_TEMPLATE_NAME, template_source)?;
    handlebars.register_helper(
        "sort_versions",
        Box::new(self::sort_versions_helper::sort_versions),
    );
    handlebars.register_helper("reverse", Box::new(self::reverse_helper::ReverseHelper));
    Ok(handlebars)
}
