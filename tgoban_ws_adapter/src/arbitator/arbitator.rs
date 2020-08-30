use tgoban_gtp::{Command, Color, CommandResult, Move, GenMoveResult};

use super::adaptor::Adaptor;

struct Arbitator {
}


pub async fn run(mut player_1: impl Adaptor + Unpin + Send, mut player_2: impl Adaptor + Unpin + Send) {
    player_1.send_command(Command::BoardSize(None, 19)).await;
    player_2.send_command(Command::BoardSize(None, 19)).await;

    player_1.send_command(Command::Komi(None, 6.5)).await;
    player_2.send_command(Command::Komi(None, 6.5)).await;

    loop {
        let command_result = player_1.send_command(Command::GenMove(None, Color::Black)).await;
        let command_result = command_result.unwrap();

        match command_result {
            CommandResult::GenMove(id, mov) => {
                match mov {
                    GenMoveResult::Resign => {
                        panic!("Not implemented");
                    },
                    GenMoveResult::Move(vertex) => {
                        player_2.send_command(
                            Command::Play(None, Move {
                                color: Color::White,
                                vertex: vertex,
                            })
                        ).await;
                    }
                }
            },
            _ => {
                panic!("Not expect result");
            },
        };

        let command_result = player_2.send_command(Command::GenMove(None, Color::White)).await;
        let command_result = command_result.unwrap();

        match command_result {
            CommandResult::GenMove(id, mov) => {
                match mov {
                    GenMoveResult::Resign => {
                        panic!("Not implemented");
                    },
                    GenMoveResult::Move(vertex) => {
                        player_1.send_command(
                            Command::Play(None, Move {
                                color: Color::Black,
                                vertex: vertex,
                            })
                        ).await;
                    }
                }
            },
            _ => {
                panic!("Not expected result");
            },
        };
    }
}
