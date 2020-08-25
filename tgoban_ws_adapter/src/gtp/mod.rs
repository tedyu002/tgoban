mod command;

use tokio::io::{AsyncWrite, AsyncRead};
use tokio::io::{AsyncReadExt, AsyncBufReadExt};

pub struct Gtp <R, W> {
    reader: R,
    writer: W,
}

pub(crate) fn build
(
    reader: impl AsyncRead + AsyncReadExt + AsyncBufReadExt + std::marker::Unpin,
    writer: impl AsyncWrite
)
-> Gtp<impl AsyncRead, impl AsyncWrite>
{
    Gtp {
        reader: reader,
        writer: writer,
    }
}

impl<R: AsyncRead + AsyncReadExt + AsyncBufReadExt + std::marker::Unpin, W: AsyncWrite> Gtp<R, W> {
    pub async fn read(&mut self) -> String {
        let mut line = "".to_string();
        self.reader.read_line(&mut line).await;

        return "".to_string();
    }
}
