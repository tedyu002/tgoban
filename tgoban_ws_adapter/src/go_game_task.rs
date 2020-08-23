use std::future::Future;

use tokio::sync::mpsc;

use tgoban_ws_adapter::{BOARD_SIZE, KOMI_DEFAULT};

use go_game_engine::{Location, GoGameEngine, ChessType, Player, GameStatus};

use tgoban_ws_protocol as protocol;

pub(crate) enum ToGo {
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

pub(crate) enum FromGo {
    SGF(String),
    Board(Vec<protocol::ChessType>),
    BelongBoard(Vec<protocol::Belong>),
    Status(GameStatus),
    Score((f64, f64)),
    GameInfo(protocol::GameInfo),
}

pub(crate) fn start() -> (impl Future<Output = ()>, mpsc::Sender<ToGo>, mpsc::Receiver<FromGo>) {
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

