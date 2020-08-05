use go_game_engine::{GoGameEngine, Location, MoveError};

use std::io;

fn main() {
    let mut game = GoGameEngine::new(19);

    loop {
        println!("{}", game);

        let mut location = String::new();

        io::stdin()
            .read_line(&mut location)
            .expect("Failed to read line");

        let location: Location = match location.trim().parse() {
            Ok(location) => location,
            Err(_) => {
                println!("Parse error");
                continue;
            },
        };

        println!("Location: {} {}", location.alphabet, location.digit);

        match game.make_move(location) {
            Ok(()) => {
            },
            Err(err) => {
                match err {
                    MoveError::Exist(location) => {
                        println!("Cannot be placed at {} {} since exist.", location.alphabet, location.digit);
                    },
                    MoveError::NoLiberty(location) => {
                        println!("Cannot be placed at {} {} since no liberty", location.alphabet, location.digit);
                    },
                    MoveError::NoMove => {
                        println!("No chess");
                    }
                }
            },
        };
    }
}
