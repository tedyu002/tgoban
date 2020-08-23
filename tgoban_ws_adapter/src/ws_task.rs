use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use tokio::net::{TcpStream};
use tokio::sync::mpsc;

use tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use crate::go_game_task::{FromGo, ToGo};

use go_game_engine::{Location};

use tgoban_ws_protocol as protocol;

pub(crate) async fn start(
    mut ws_stream: WebSocketStream<TcpStream>,
    mut go_sender: mpsc::Sender<ToGo>,
    mut go_receiver: mpsc::Receiver<FromGo>
) {
    loop {
        tokio::select! {
            v = ws_stream.next() => {
                match v {
                    Some(Ok(message)) => {
                        handle_ws_receive(message, &mut ws_stream, &mut go_sender).await;
                    },
                    _ => break,
                }
            },
            v = go_receiver.recv() => {
                match v {
                    Some(message) => {
                        handle_go_receive(message, &mut ws_stream).await;
                        continue;
                    },
                    _ => break,
                }
            }
        };

        /* Refresh board */
        match go_sender.send(ToGo::GetBoard).await {
            Ok(_) => {},
            Err(_) => break,
        };

        /* Refresh belong */
        match go_sender.send(ToGo::GetBelong).await {
            Ok(_) => {},
            Err(_) => break,
        };

        /* Force refresh game info */
        match go_sender.send(ToGo::GetGameInfo).await {
            Ok(_) => {},
            Err(_) => break,
        };
    };
}

async fn handle_ws_receive(
    message: Message,
    ws_stream: &mut WebSocketStream<TcpStream>,
    go_sender: &mut mpsc::Sender<ToGo>,
) {
    if let Message::Text(text) = message {
        let action: Result<protocol::Action, _> = serde_json::from_str(&text);

        if let Err(_) = action {
            return;
        }

        let action = action.unwrap();

        match action {
            protocol::Action::Play(location) => {
                let location = Location {
                    alphabet: location.alphabet,
                    digit: location.digit,
                };

                match go_sender.send(ToGo::Play(location)).await {
                    Ok(_) => {},
                    Err(_) => return,
                };
            },
            protocol::Action::Back => {
                match go_sender.send(ToGo::Back).await {
                    Ok(_) => {},
                    Err(_) => return,
                };
            },
            protocol::Action::Pass => {
                match go_sender.send(ToGo::Pass).await {
                    Ok(_) => {},
                    Err(_) => return,
                };
            },
            protocol::Action::GetSGF => {
                match go_sender.send(ToGo::GetSGF).await {
                    Ok(_) => {},
                    Err(_) => return,
                }
            },
            protocol::Action::Refresh => {
                /* Do nothing */
            },
        };

    } else if let Message::Ping(msg) = message {
        match ws_stream.send(Message::Pong(msg)).await {
            Ok(_) => {},
            Err(_) => return,
        };
    }
}

async fn handle_go_receive(
    message: FromGo,
    ws_stream: &mut WebSocketStream<TcpStream>,
) {
    match message {
        FromGo::SGF(sgf) => {
            let res = ws_stream.send(Message::Text(serde_json::to_string_pretty(&protocol::Command::Sgf(
                sgf
            )).unwrap())).await;

            match res {
                Ok(_) => {},
                Err(_) => return,
            }
        },
        FromGo::Board(board) => {
            let command = protocol::Command::Set(board);
            match ws_stream.send(Message::Text(serde_json::to_string_pretty(&command).unwrap())).await {
                Ok(_) => {},
                Err(_) => return,
            };
        },
        FromGo::BelongBoard(board) => {
            let command = protocol::Command::SetBelong(board);
            match ws_stream.send(Message::Text(serde_json::to_string_pretty(&command).unwrap())).await {
                Ok(_) => {},
                Err(_) => return,
            };
        },
        FromGo::GameInfo(game_info) => {
            let res = ws_stream.send(Message::Text(serde_json::to_string_pretty(&protocol::Command::SetGameInfo(
                game_info
            )).unwrap())).await;

            match res {
                Ok(_) => {},
                Err(_) => return,
            }
        },
    };
}
