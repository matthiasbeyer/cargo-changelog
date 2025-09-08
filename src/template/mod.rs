use handlebars::Handlebars;

use crate::error::Error;

mod group_by_helper;
mod indent_helper;
mod reverse_helper;
mod sort_versions_helper;

pub fn new_handlebars(template_source: &str) -> Result<Handlebars<'_>, Error> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars
        .register_template_string(crate::consts::INTERNAL_TEMPLATE_NAME, template_source)
        .map_err(Box::new)?;
    handlebars.register_helper(
        "sort_versions",
        Box::new(self::sort_versions_helper::sort_versions),
    );
    handlebars.register_helper("reverse", Box::new(self::reverse_helper::ReverseHelper));
    handlebars.register_helper(
        "group_by_header",
        Box::new(self::group_by_helper::GroupByHelper),
    );
    handlebars.register_helper("indent", Box::new(self::indent_helper::IndentHelper));
    Ok(handlebars)
}
