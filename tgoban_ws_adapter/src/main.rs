use std::net::SocketAddr;
use std::future::Future;

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

use go_game_engine::{Location, GoGameEngine, ChessType, Player, GameStatus};

use tgoban_ws_protocol as protocol;

const BOARD_SIZE: u8 = 19;
const KOMI_DEFAULT: f64 = 6.5;

async fn handle_connection(raw_stream: TcpStream, _addr: SocketAddr) {
    let ws_stream: WebSocketStream<TcpStream> = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (go_game_task, go_sender, go_receiver) = start_go_game_task();
    let ws_task = start_ws_task(ws_stream, go_sender, go_receiver);

    tokio::spawn(go_game_task);
    tokio::spawn(ws_task);
}

async fn start_ws_task(
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

enum ToGo {
    Play(Location),
    Back,
    Pass,
    GetSGF,
    GetBoard,
    GetBelong,
    GetStatus,
    GetGameInfo,
    GetScore,
}

enum FromGo {
    SGF(String),
    Board(Vec<protocol::ChessType>),
    BelongBoard(Vec<protocol::Belong>),
    Status(GameStatus),
    Score((f64, f64)),
    GameInfo(protocol::GameInfo),
}

fn start_go_game_task() -> (impl Future<Output = ()>, mpsc::Sender<ToGo>, mpsc::Receiver<FromGo>) {
    let (incoming_write, incoming_read) = mpsc::channel::<ToGo>(32 /* TODO */);
    let (outgoing_write, outgoing_read) = mpsc::channel::<FromGo>(32 /* TODO */);

    let task = async move {
        let mut receiver = incoming_read;
        let mut sender = outgoing_write;
        let mut go_game = GoGameEngine::new(BOARD_SIZE, KOMI_DEFAULT);

        loop {
            let recv_command = match receiver.recv().await {
                None => break,
                Some(command) => command,
            };

            match recv_command {
                ToGo::Play(location) => {
                    match go_game.get_status() {
                        GameStatus::Playing => {
                            match go_game.make_move(location) {
                                Ok(_chess_change) => {},
                                Err(_) => {
                                    continue;
                                },
                            };
                        },
                        GameStatus::Scoring => {
                            go_game.toggle(location);
                        },
                    };
                },
                ToGo::Back => {
                    go_game.regret();
                },
                ToGo::Pass => {
                    go_game.pass();
                },
                ToGo::GetSGF => {
                    match sender.send(FromGo::SGF(go_game.to_sgf())).await {
                        Ok(_) => {},
                        Err(_) => break,
                    }
                },
                ToGo::GetBoard => {
                    let mut board: Vec<protocol::ChessType> = Vec::new();
                    for x in 0..BOARD_SIZE {
                        for y in 0..BOARD_SIZE {
                            let location = Location {
                                alphabet: x,
                                digit: y,
                            };

                            let mut is_dead = false;
                            if go_game.get_status() == GameStatus::Scoring && !go_game.is_alive(location) {
                                is_dead = true;
                            }
                            let chess = match go_game.get_chess(location) {
                                ChessType::Black => match is_dead {
                                    false => protocol::ChessType::BlackLive,
                                    true => protocol::ChessType::BlackDead,
                                },
                                ChessType::White => match is_dead {
                                    false => protocol::ChessType::WhiteLive,
                                    true => protocol::ChessType::WhiteDead,
                                },
                                ChessType::None => protocol::ChessType::None,
                            };

                            board.push(chess);
                        }
                    }

                    match sender.send(FromGo::Board(board)).await {
                        Ok(_) => {},
                        Err(_) => break,
                    }
                },
                ToGo::GetBelong => {
                    let mut belong_board: Vec<protocol::Belong> = Vec::new();

                    for alphabet in 0..BOARD_SIZE {
                        for digit in 0..BOARD_SIZE {
                            belong_board.push(match go_game.get_belong(Location {
                                alphabet,
                                digit,
                            }) {
                                None => protocol::Belong::None,
                                Some(player) => {
                                    match player {
                                        Player::Black => {
                                            protocol::Belong::Black
                                        },
                                        Player::White => {
                                            protocol::Belong::White
                                        },
                                    }
                                }
                            });
                        }
                    }
                    match sender.send(FromGo::BelongBoard(belong_board)).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                },
                ToGo::GetStatus => {
                    match sender.send(FromGo::Status(go_game.get_status())).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                },
                ToGo::GetScore => {
                    match sender.send(FromGo::Score(go_game.get_score())).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                },
                ToGo::GetGameInfo => {
                    let game_info = protocol::GameInfo {
                        steps: go_game.steps(),
                        playing: match go_game.player() {
                            Player::Black => 'B',
                            Player::White => 'W',
                        },
                        komi: go_game.komi(),
                        capture: [go_game.get_capture(&Player::Black), go_game.get_capture(&Player::White)],
                    };

                    match sender.send(FromGo::GameInfo(game_info)).await {
                        Ok(_) => {},
                        Err(_) => break,
                    };
                },
            }
        };
    };

    return (task, incoming_write, outgoing_read);
}

#[tokio::main]
async fn main() {
    let mut server = TcpListener::bind("127.0.0.1:8088").await.expect("Failed to bind");

    while let Ok((stream, addr)) = server.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
