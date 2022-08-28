mod new_command;
pub use self::new_command::NewCommand;

pub trait Command {
    fn execute(self) -> miette::Result<()>;
}
