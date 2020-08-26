mod command;

use tokio::io::{AsyncWrite, AsyncRead};
use tokio::io::BufReader;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

pub struct Gtp <R: AsyncRead + std::marker::Unpin, W: AsyncWrite>{
    reader: BufReader<R>,
    writer: W,
}

impl<R: AsyncRead + std::marker::Unpin, W: AsyncWrite> Gtp<R, W> {
    pub fn new (
        reader: R,
        writer: W,
    )
    -> Gtp<R, W>
    {
        Gtp {
            reader: BufReader::new(reader),
            writer: writer,
        }
    }

    pub async fn read_command(&mut self) -> Result<command::Command, ()> {
        let mut line = "".to_string();

        let command: command::Command = loop {
            let result = self.reader.read_line(&mut line).await;

            if let Err(_) = result {
                return Err(());
            }

            if line.starts_with('#') {
                continue;
            }

            let parse_result: Result<command::Command, _> = line.parse();

            match parse_result {
                Ok(command) => command,
                Err(_) => return Err(()),
            };
        };

        return Ok(command);
    }
}
