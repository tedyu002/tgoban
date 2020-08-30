use std::future::Future;

use tokio::io::{AsyncWrite, AsyncRead};
use tokio::io::BufReader;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt};

use tgoban_gtp::{Command, CommandResult};

use super::adaptor::Adaptor;

pub struct AsyncAdaptor <R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send>{
    reader: BufReader<R>,
    writer: W,
}

impl<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send> AsyncAdaptor<R, W> {
    pub fn new (
        reader: R,
        writer: W,
    )
    -> AsyncAdaptor<R, W>
    {
        AsyncAdaptor {
            reader: BufReader::new(reader),
            writer: writer,
        }
    }
}


impl<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send> Adaptor for AsyncAdaptor<R, W> {
    fn send_command<'a>(&'a mut self, command: Command) -> Box<dyn Future<Output = Result<CommandResult, ()>> + Unpin + Send + 'a> {
        let future = async move {
            let mut lines = "".to_string();

            self.writer.write(format!("{}\n", command.to_string()).as_bytes()).await;

            loop {
                let mut line = "".to_string();
                let result = self.reader.read_line(&mut line).await;

                match result {
                    Ok(size) => {
                        if size == 1 {
                            break;
                        }
                        lines.push_str(&line);
                    },
                    Err(_) => {
                        return Err(());
                    },
                };
            }

            return command.parse_result(&lines);
        };

        Box::new(Box::pin(future))
    }
}
