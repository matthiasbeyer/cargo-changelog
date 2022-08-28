use crate::format::Format;

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct NewCommand {
    interactive: bool,
    edit: bool,
    format: Format,
}

impl crate::command::Command for NewCommand {
    fn execute(self) -> miette::Result<()> {
        unimplemented!()
    }
}
