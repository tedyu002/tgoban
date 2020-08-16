use std::thread;
use websocket::sync::Server;
use websocket::sender::Writer;
use websocket::OwnedMessage;
use websocket::result::WebSocketError;
use std::sync::mpsc;

use go_game_engine::{Location, GoGameEngine, ChessType, Player, GameStatus};

use tgoban_ws_protocol as protocol;

const BOARD_SIZE: u8 = 19;
const KOMI_DEFAULT: f64 = 6.5;

fn handle(message: OwnedMessage, sender: &mut Writer<std::net::TcpStream>,
    go_sender: &mpsc::Sender<ToGo>,
    go_receiver: &mpsc::Receiver<FromGo>
) -> Result<(), WebSocketError> {
    if let OwnedMessage::Text(text) = message {
        let action: Result<protocol::Action, _> = serde_json::from_str(&text);

        if let Ok(action) = action {
            let mut draw_chess = true;
            match action {
                protocol::Action::Play(location) => {
                    let location = Location {
                        alphabet: location.alphabet,
                        digit: location.digit,
                    };

                    go_sender.send(ToGo::Play(location));
                },
                protocol::Action::Back => {
                    go_sender.send(ToGo::Back);
                },
                protocol::Action::Pass => {
                    go_sender.send(ToGo::Pass);
                },
                protocol::Action::GetSGF => {
                    go_sender.send(ToGo::GetSGF);

                    let recv_sgf = go_receiver.recv().unwrap();

                    match recv_sgf {
                        FromGo::SGF(sgf) => {
                            sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&protocol::Command::Sgf(
                                sgf
                            )).unwrap()))?;
                            return Ok(());
                        },
                        _ => {
                            panic!("Not expect result");
                        }
                    }
                },
                protocol::Action::Refresh => {
                    /* Do nothing */
                },
            };

            { /* Draw Chess */
                go_sender.send(ToGo::GetBoard);

                let receiver = go_receiver.recv().unwrap();

                match receiver {
                    FromGo::Board(board) => {
                        let command = protocol::Command::Set(board);
                        sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
                    },
                    _ => {
                        panic!("Not expected type");
                    },
                };
            }

            {
                go_sender.send(ToGo::GetBelong);

                let receiver = go_receiver.recv().unwrap();

                match receiver {
                    FromGo::BelongBoard(board) => {
                        let command = protocol::Command::SetBelong(board);
                        sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
                    },
                    _ => {
                        panic!("Not expected type");
                    },
                };
            }

            { /* Set Status */
                let status = {
                    go_sender.send(ToGo::GetStatus);

                    match go_receiver.recv().unwrap() {
                        FromGo::Status(status) => status,
                        _ => panic!("Not expected type"),
                    }
                };

                if status == GameStatus::Scoring {
                    /* Set Score info */
                    let score = {
                        go_sender.send(ToGo::GetScore);

                        match go_receiver.recv().unwrap() {
                            FromGo::Score(score) => score,
                            _ => panic!("Not expected type"),
                        }
                    };
                    let command = protocol::Command::SetScoring(score);
                    sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
                } else {
                    let command = protocol::Command::SetScoring((0.0, 0.0));
                    sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
                }

            }
        }
    } else if let OwnedMessage::Ping(msg) = message {
        sender.send_message(&OwnedMessage::Pong(msg))?;
    }


    { /* Draw Game Info */
        let game_info = {
            go_sender.send(ToGo::GetGameInfo);

            let game_info = match go_receiver.recv().unwrap() {
                FromGo::GameInfo(game_info) => game_info,
                _ => panic!("Not expected type"),
            };
            game_info
        };
        sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&protocol::Command::SetGameInfo(
            game_info
        )).unwrap()))?;
    }

    return Ok(());
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

fn start_go_game_thread() -> (mpsc::Sender<ToGo>, mpsc::Receiver<FromGo>) {
    let (incoming_write, incoming_read) = mpsc::channel::<ToGo>();
    let (outgoing_write, outgoing_read) = mpsc::channel::<FromGo>();

    thread::spawn( move || {
        let receiver = incoming_read;
        let sender = outgoing_write;
        let mut go_game = GoGameEngine::new(BOARD_SIZE, KOMI_DEFAULT);

        loop {
            let recv_command = match receiver.recv() {
                Err(_) => break,
                Ok(command) => command,
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
                    sender.send(FromGo::SGF(go_game.to_sgf()));
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

                    sender.send(FromGo::Board(board));
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
                    sender.send(FromGo::BelongBoard(belong_board));
                },
                ToGo::GetStatus => {
                    sender.send(FromGo::Status(go_game.get_status()));
                },
                ToGo::GetScore => {
                    sender.send(FromGo::Score(go_game.get_score()));
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

                    sender.send(FromGo::GameInfo(game_info));
                },
            }
        }
    });

    return (incoming_write, outgoing_read);
}

fn main() {
    let server = Server::bind("127.0.0.1:8088").unwrap();

    for request in server.filter_map(Result::ok) {
        thread::spawn(move || {
            let client = request.accept().unwrap();

            let (mut ws_receiver, mut ws_sender) = client.split().unwrap();

            let (go_sender, go_receiver) = start_go_game_thread();

            for message in ws_receiver.incoming_messages() {
                let message = message.unwrap();

                let res = handle(message, &mut ws_sender, &go_sender, &go_receiver);

                match res {
                    Err(_) => {
                        break;
                    },
                    Ok(_) => {
                        continue;
                    },
                }
            }
        });
    }
}
