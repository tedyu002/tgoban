use go_board::{GoBoard, ChessType, MoveError};
use std::io;

fn main() {
    let mut board = GoBoard::new(19);

    loop {
        let mut location = String::new();

        io::stdin()
            .read_line(&mut location)
            .expect("Failed to read line");

        let location: go_board::Location = match location.trim().parse() {
            Ok(location) => location,
            Err(_) => {
                println!("Parse error");
                continue;
            },
        };

        println!("Location: {} {}", location.x, location.y);

        match board.make_move(ChessType::Black, location) {
            Ok(chess_change) => {
                println!("Place at {} {}", chess_change.at.location.x, chess_change.at.location.y);
            },
            Err(err) => {
                match err {
                    MoveError::Exist(location) => {
                        println!("Cannot be placed at {} {} since exist.", location.x, location.y);
                    },
                    MoveError::NoLiberty(location) => {
                        println!("Cannot be placed at {} {} since no liberty", location.x, location.y);
                    },
                    MoveError::NoMove => {
                        println!("No chess");
                    }
                }
            },
        };
    }
}
