use std::future::Future;

use tgoban_gtp::{Command, CommandResult};

pub trait Adaptor {
    fn send_command<'a>(&'a mut self, command: Command) -> Box<dyn Future<Output = Result<CommandResult, ()>> + Unpin + Send + 'a>;
}
