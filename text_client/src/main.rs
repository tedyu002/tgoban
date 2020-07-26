use go_board::MoveError;
use go_game_engine::GoGameEngine;
use std::io;

fn main() {
    let mut game = GoGameEngine::new(19);

    loop {
        println!("{}", game);

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

        match game.make_move(location) {
            Ok(()) => {
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
