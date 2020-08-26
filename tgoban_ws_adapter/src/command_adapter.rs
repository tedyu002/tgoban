use std::process::Stdio;

use tokio::process::{Command};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::io::{AsyncWriteExt};
use tokio::runtime;

use crate::gtp;

pub fn spawn_command() {
    let handle = async move {
        let child = Command::new("gnugo")
            .arg("--mode").arg("gtp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        let child = match child {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut stdin = child.stdin.unwrap();
        let mut stdout = child.stdout.unwrap();

        let mut gtp_engine = gtp::Gtp::new(stdout, stdin);

        loop {
            let command = gtp_engine.read_command().await;
        };
    };

    tokio::spawn(handle);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
    }
}
