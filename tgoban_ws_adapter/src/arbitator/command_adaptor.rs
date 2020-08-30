use std::future::Future;
use std::process::Stdio;

use tokio::process::{Command, Child, ChildStdin, ChildStdout};
use tokio::io::{AsyncRead, AsyncBufReadExt, AsyncReadExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use tgoban_gtp::Command as GtpCommand;
use tgoban_gtp::CommandResult as GtpCommandResult;

use super::adaptor::Adaptor;
use super::async_adaptor::AsyncAdaptor;

pub struct CommandAdaptor {
    child: Child,
    async_adaptor: AsyncAdaptor<ChildStdout, ChildStdin>,
}

impl CommandAdaptor {
    pub fn new (
        mut child: Child,
    )
    -> CommandAdaptor
    {
        let stdin = child.stdin.unwrap();
        let stdout = child.stdout.unwrap();
        child.stdin = None;
        child.stdout = None;

        CommandAdaptor {
            async_adaptor: AsyncAdaptor::new(stdout, stdin),
            child: child,
        }
    }
}

impl Adaptor for CommandAdaptor {
    fn send_command<'a>(&'a mut self, command: GtpCommand) -> Box<Future<Output = Result<GtpCommandResult, ()>> + Unpin + Send + 'a> {
        let future = async move {
            let mut line = "".to_string();

            return self.async_adaptor.send_command(command).await;
        };

        Box::new(Box::pin(future))
    }
}

pub fn spawn_command() -> CommandAdaptor {
    let child = Command::new("gnugo")
        .arg("--mode").arg("gtp")
        .arg("--level").arg("20")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let mut child = child.unwrap();

    CommandAdaptor::new(child)
}
