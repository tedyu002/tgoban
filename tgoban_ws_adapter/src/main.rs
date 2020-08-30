//mod command_adapter;
mod arbitator;

use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};

use tokio_tungstenite::WebSocketStream;

pub const BOARD_SIZE: u8 = 19;
pub const KOMI_DEFAULT: f64 = 6.5;


async fn handle_connection(raw_stream: TcpStream, _addr: SocketAddr) {
    let ws_stream: WebSocketStream<TcpStream> = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let ws_adaptor = arbitator::ws_adaptor::WsAdaptor::new(ws_stream);
    let command_adaptor = arbitator::command_adaptor::spawn_command();

    arbitator::arbitator::run(ws_adaptor, command_adaptor).await;
}

#[tokio::main]
async fn main() {
    let mut server = TcpListener::bind("127.0.0.1:8088").await.expect("Failed to bind");

    while let Ok((stream, addr)) = server.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
