mod go_game_task;
mod ws_task;
mod command_adapter;
mod gtp;

use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};

use tokio_tungstenite::WebSocketStream;

pub const BOARD_SIZE: u8 = 19;
pub const KOMI_DEFAULT: f64 = 6.5;


async fn handle_connection(raw_stream: TcpStream, _addr: SocketAddr) {
    let ws_stream: WebSocketStream<TcpStream> = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (go_game_task, go_sender, go_receiver) = go_game_task::start();
    let ws_task = ws_task::start(ws_stream, go_sender, go_receiver);

    tokio::spawn(go_game_task);
    tokio::spawn(ws_task);
    command_adapter::spawn_command();
}

#[tokio::main]
async fn main() {
    let mut server = TcpListener::bind("127.0.0.1:8088").await.expect("Failed to bind");

    while let Ok((stream, addr)) = server.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
