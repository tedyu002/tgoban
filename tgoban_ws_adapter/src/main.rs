use std::thread;
use websocket::sync::Server;
use websocket::sender::Writer;
use websocket::OwnedMessage;
use websocket::result::WebSocketError;

use go_game_engine::{Location, GoGameEngine, ChessType, Player, GameStatus};

use tgoban_ws_protocol as protocol;

const BOARD_SIZE: u8 = 19;
const KOMI_DEFAULT: f64 = 6.5;

fn handle(message: OwnedMessage, sender: &mut Writer<std::net::TcpStream>, go_game: &mut GoGameEngine) -> Result<(), WebSocketError> {
    if let OwnedMessage::Text(text) = message {
        let action: Result<protocol::Action, _> = serde_json::from_str(&text);

        let original_status = go_game.get_status();

        if let Ok(action) = action {
            let mut draw_chess = true;
            match action {
                protocol::Action::Play(location) => {
                    let location = Location {
                        alphabet: location.alphabet,
                        digit: location.digit,
                    };
                    match go_game.get_status() {
                        GameStatus::Playing => {
                            match go_game.make_move(location) {
                                Ok(_chess_change) => {},
                                Err(_) => {
                                    draw_chess = false;
                                },
                            };
                        },
                        GameStatus::Scoring => {
                            go_game.toggle(location);
                        },
                    };
                },
                protocol::Action::Back => {
                    go_game.regret();
                },
                protocol::Action::Pass => {
                    go_game.pass();
                    draw_chess = false;
                },
                protocol::Action::GetSGF => {
                    sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&protocol::Command::Sgf(
                        go_game.to_sgf()
                    )).unwrap()))?;
                    return Ok(());
                },
                protocol::Action::Refresh => {
                    /* Do nothing */
                },
            };

            if draw_chess { /* Draw Chess */
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

                let command = protocol::Command::Set(board);
                sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
            }

            if original_status == GameStatus::Scoring || go_game.get_status() == GameStatus::Scoring {
                /* Draw Belong */
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
                let command = protocol::Command::SetBelong(belong_board);
                sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
            }

            if go_game.get_status() == GameStatus::Scoring {
                /* Set Score info */
                let command = protocol::Command::SetScoring(go_game.get_score());
                sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
            } else {
                let command = protocol::Command::SetScoring((0.0, 0.0));
                sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&command).unwrap()))?;
            }
        }
    } else if let OwnedMessage::Ping(msg) = message {
        sender.send_message(&OwnedMessage::Pong(msg))?;
    }
    sender.send_message(&OwnedMessage::Text(serde_json::to_string_pretty(&protocol::Command::SetGameInfo(protocol::GameInfo {
        steps: go_game.steps(),
        playing: match go_game.player() {
            Player::Black => 'B',
            Player::White => 'W',
        },
        komi: go_game.komi(),
        capture: [go_game.get_capture(&Player::Black), go_game.get_capture(&Player::White)],
    })).unwrap()))?;

    return Ok(());
}

fn main() {
    let server = Server::bind("127.0.0.1:8088").unwrap();

    for request in server.filter_map(Result::ok) {
        thread::spawn(move || {
            let client = request.accept().unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();
            let mut go_game = GoGameEngine::new(BOARD_SIZE, KOMI_DEFAULT);

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                let res = handle(message, &mut sender, &mut go_game);

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
