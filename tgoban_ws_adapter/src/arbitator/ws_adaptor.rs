use std::future::Future;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use tokio::net::{TcpStream};
use tokio::sync::mpsc;

use tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use go_game_engine::{Location};

use tgoban_gtp::{Command, CommandResult};
use super::adaptor::Adaptor;

pub struct WsAdaptor {
    ws_stream: WebSocketStream<TcpStream>,
}

impl WsAdaptor {
    pub fn new(ws_stream: WebSocketStream<TcpStream>) -> WsAdaptor {
        WsAdaptor {
            ws_stream,
        }
    }
}

impl Adaptor for WsAdaptor {
    fn send_command<'a>(&'a mut self, command: Command) -> Box<dyn Future<Output = Result<CommandResult, ()>> + Unpin + Send + 'a> {
        let future = async move {
            let res = self.ws_stream.send(Message::Text(command.to_string())).await;

            loop {
                let message = self.ws_stream.next().await;

                if let None = message {
                    return Err(());
                };

                let message = message.unwrap().unwrap();

                match message {
                    Message::Text(text) => {
                        return command.parse_result(&text);
                    }
                    Message::Ping(payload) => {
                        self.ws_stream.send(Message::Pong(payload)).await;
                    },
                    Message::Close(close) => {
                        return Err(());
                    }
                    _ => {
                        return Err(());
                    }
                };
            }

            return Err(());
        };

        Box::new(Box::pin(future))
    }
}
