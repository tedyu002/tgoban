use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use tokio::net::{TcpStream};
use tokio::sync::mpsc;

use tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use crate::go_game_task::{FromGo, ToGo};

use go_game_engine::{Location, GameStatus};

use tgoban_ws_protocol as protocol;

pub(crate) async fn start(
    mut ws_stream: WebSocketStream<TcpStream>,
    mut go_sender: mpsc::Sender<ToGo>,
    mut go_receiver: mpsc::Receiver<FromGo>
) {
    loop {
        let message = ws_stream.next().await;

        if let None = message {
            break;
        }

        let message = message.unwrap();

        if let Err(_) = message {
            break;
        }

        let message = message.unwrap();

        if let Message::Text(text) = message {
            let action: Result<protocol::Action, _> = serde_json::from_str(&text);

            if let Err(_) = action {
                continue;
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
                        Err(_) => break,
                    };
                },
                protocol::Action::Back => {
                    match go_sender.send(ToGo::Back).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                },
                protocol::Action::Pass => {
                    match go_sender.send(ToGo::Pass).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                },
                protocol::Action::GetSGF => {
                    match go_sender.send(ToGo::GetSGF).await {
                        Ok(_) => {},
                        Err(_) => break,
                    }

                    let recv_sgf = go_receiver.recv().await.unwrap();

                    match recv_sgf {
                        FromGo::SGF(sgf) => {
                            let res = ws_stream.send(Message::Text(serde_json::to_string_pretty(&protocol::Command::Sgf(
                                sgf
                            )).unwrap())).await;

                            match res {
                                Ok(_) => {},
                                Err(_) => break,
                            }
                            continue;
                        },
                        _ => {
                            panic!("Not expect result");
                        }
                    };
                },
                protocol::Action::Refresh => {
                    /* Do nothing */
                },
            };

            { /* Draw Chess */
                match go_sender.send(ToGo::GetBoard).await {
                    Ok(_) => {},
                    Err(_) => break,
                }

                let receiver = go_receiver.recv().await.unwrap();

                match receiver {
                    FromGo::Board(board) => {
                        let command = protocol::Command::Set(board);
                        match ws_stream.send(Message::Text(serde_json::to_string_pretty(&command).unwrap())).await {
                            Ok(_) => {},
                            Err(_) => break,
                        };
                    },
                    _ => {
                        panic!("Not expected type");
                    },
                };
            }

            {
                match go_sender.send(ToGo::GetBelong).await {
                    Ok(_) => {},
                    Err(_) => break,
                };

                let receiver = go_receiver.recv().await.unwrap();

                match receiver {
                    FromGo::BelongBoard(board) => {
                        let command = protocol::Command::SetBelong(board);
                        match ws_stream.send(Message::Text(serde_json::to_string_pretty(&command).unwrap())).await {
                            Ok(_) => {},
                            Err(_) => break,
                        }
                    },
                    _ => {
                        panic!("Not expected type");
                    },
                };
            }

            { /* Set Status */
                let status = {
                    match go_sender.send(ToGo::GetStatus).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };

                    match go_receiver.recv().await.unwrap() {
                        FromGo::Status(status) => status,
                        _ => panic!("Not expected type"),
                    }
                };

                if status == GameStatus::Scoring {
                    /* Set Score info */
                    let score = {
                        match go_sender.send(ToGo::GetScore).await {
                            Ok(_) => {},
                            Err(_) => break,
                        };

                        match go_receiver.recv().await.unwrap() {
                            FromGo::Score(score) => score,
                            _ => panic!("Not expected type"),
                        }
                    };
                    let command = protocol::Command::SetScoring(score);
                    match ws_stream.send(Message::Text(serde_json::to_string_pretty(&command).unwrap())).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                } else {
                    let command = protocol::Command::SetScoring((0.0, 0.0));
                    match ws_stream.send(Message::Text(serde_json::to_string_pretty(&command).unwrap())).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                }

            }
        } else if let Message::Ping(msg) = message {
            match ws_stream.send(Message::Pong(msg)).await {
                Ok(_) => {},
                Err(_) => break,
            };
        }

        { /* Draw Game Info */
            let game_info = {
                match go_sender.send(ToGo::GetGameInfo).await {
                    Ok(_) => {},
                    Err(_) => break,
                };

                let game_info = match go_receiver.recv().await.unwrap() {
                    FromGo::GameInfo(game_info) => game_info,
                    _ => panic!("Not expected type"),
                };
                game_info
            };
            let res = ws_stream.send(Message::Text(serde_json::to_string_pretty(&protocol::Command::SetGameInfo(
                game_info
            )).unwrap())).await;

            match res {
                Ok(_) => {},
                Err(_) => break,
            };
        }
    };
}


