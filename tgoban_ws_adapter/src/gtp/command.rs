use std::num::{ParseIntError, ParseFloatError};

pub enum RawCommand {
    ProtocolVersion,
    Name,
    Version,
    KnownCommand,
    ListCommands,
    Quit,
    BoardSize,
    ClearBoard,
    Komi,
    Play,
    GenMove,
}

impl std::str::FromStr for RawCommand {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let raw = match command {
            "protocol_version" => RawCommand::ProtocolVersion,
            "name" => RawCommand::Name,
            "version" => RawCommand::Version,
            "known_command" => RawCommand::KnownCommand,
            "list_commands" => RawCommand::ListCommands,
            "quit" => RawCommand::Quit,
            "boardsize" => RawCommand::BoardSize,
            "clear_board" => RawCommand::ClearBoard,
            "komi" => RawCommand::Komi,
            "play" => RawCommand::Play,
            "genmove" => RawCommand::GenMove,
            _ => {
                return Err(())
            },
        };

        Ok(raw)
    }
}

pub enum Command {
    ProtocolVersion(Option<i32>),
    Name(Option<i32>),
    Version(Option<i32>),
    KnownCommand(Option<i32>, String),
    ListCommands(Option<i32>),
    Quit(Option<i32>),
    BoardSize(Option<i32>, i32),
    ClearBoard(Option<i32>),
    Komi(Option<i32>, f32),
    Play(Option<i32>, Move),
    GenMove(Option<i32>, Color),
}

impl std::str::FromStr for Command {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut tokens: Vec<&str> = command.split_ascii_whitespace().collect();

        if tokens.len() == 0 {
            return Err(());
        }

        let id: Result<i32, ParseIntError> = tokens[0].parse();

        let id = match id {
            Ok(val) => {
                tokens.remove(0);
                Some(val)
            },
            Err(_) => None,
        };

        if tokens.len() == 0 {
            return Err(());
        }

        let command_token = tokens[0];
        tokens.remove(0);

        let command: Command = match command_token {
            "protocol_version" => {
                Command::ProtocolVersion(id)
            },
            "name" => {
                Command::Name(id)
            },
            "version" => {
                Command::Version(id)
            },
            "known_command" => {
                if tokens.len() == 0 {
                    return Err(());
                }
                Command::KnownCommand(id, tokens[0].to_string())
            },
            "list_commands" => {
                Command::ListCommands(id)
            },
            "quit" => {
                Command::Quit(id)
            },
            "boardsize" => {
                let boardsize: Result<i32, ParseIntError> = tokens[0].parse();

                let boardsize = match boardsize {
                    Ok(size) => size,
                    Err(_) => {
                        return Err(())
                    },
                };
                Command::BoardSize(id, boardsize)
            },
            "clear_board" => {
                Command::ClearBoard(id)
            },
            "komi" => {
                if tokens.len() == 0 {
                    return Err(());
                }

                let new_komi: Result<f32, ParseFloatError> = tokens[0].parse();
                let new_komi = match new_komi {
                    Ok(komi) => komi,
                    Err(_) => return Err(()),
                };

                Command::Komi(id, new_komi)
            },
            "play" => {
                if tokens.len() != 2 {
                    return Err(());
                }

                let color: Color = match tokens[0].parse() {
                    Ok(c) => c,
                    Err(_) => return Err(()),
                };
                let vertex: Vertex = match tokens[1].parse() {
                    Ok(v) => v,
                    Err(_) => return Err(()),
                };

                Command::Play(id, Move {
                    color: color,
                    vertex: vertex,
                })
            },
            "genmove" => {
                if tokens.len() != 1 {
                    return Err(());
                }

                let color: Result<Color, ()> = tokens[0].parse();
                let color = match color{
                    Ok(color) => color,
                    Err(_) => return Err(()),
                };

                Command::GenMove(id, color)
            },
            _ => {
                return Err(())
            },
        };

        Ok(command)
    }
}

pub enum Color {
    White,
    Black,
}

impl std::str::FromStr for Color {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let lower = command.to_lowercase();

        match lower.as_str() {
            "b" | "black" => Ok(Color::Black),
            "w" | "white" => Ok(Color::White),
            _ => return Err(()),
        }
    }
}

pub enum Vertex {
    Pass,
    Coordinate(char, i32),
}

impl std::str::FromStr for Vertex {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        if command == "pass" {
            return Ok(Vertex::Pass);
        }

        let mut command:Vec<char> = command.chars().collect();

        let letter = command[0];
        if !letter.is_ascii_alphabetic() || letter == 'i'  || letter == 'I' {
            return Err(());
        }

        command.remove(0);
        if command.len() == 0 || command.len() >= 3 {
            return Err(());
        }

        let number: String = command.into_iter().collect();
        let number:i32 = match number.parse() {
            Ok(num) => num,
            Err(_) => return Err(()),
        };

        return Ok(Vertex::Coordinate(letter, number));
    }
}

pub struct Move {
    color: Color,
    vertex: Vertex,
}

impl std::str::FromStr for Move {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let tokens: Vec<&str> = command.split_ascii_whitespace().collect();

        if tokens.len() != 2 {
            return Err(());
        }

        let color: Color = match tokens[0].parse() {
            Ok(c) => c,
            Err(_) => return Err(()),
        };

        let vertex: Vertex = match tokens[1].parse() {
            Ok(v) => v,
            Err(_) => return Err(()),
        };

        Ok(Move {
            color: color,
            vertex: vertex,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_protocol_version() {
        let command: Result<Command, _> = "protocol_version".parse();

        match command {
            Ok(command) => match command {
                Command::ProtocolVersion(id) => {
                    match id {
                        None => (),
                        _ => assert!(false, "The id is exist but it should not be"),
                    }
                },
                _ => assert!(false, "The parse command is not protocol_version"),
            },
            _ => assert!(false, "The parse result is failed"),
        };

        let command: Result<Command, _> = "423 protocol_version".parse();

        match command {
            Ok(command) => match command {
                Command::ProtocolVersion(id) => {
                    match id {
                        Some(num) => assert_eq!(num, 423),
                        None => assert!(false, "The id is not exist"),
                    }
                },
                _ => assert!(false, "The parse command is not protocol_version"),
            },
            _ => assert!(false, "The parse result is failed"),
        };
    }

    #[test]
    pub fn test_name() {
        let command: Result<Command, _> = "name".parse();

        match command {
            Ok(command) => match command {
                Command::Name(id) => (),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }
  
    #[test]
    pub fn test_version() {
        let command: Result<Command, _> = "version".parse();

        match command {
            Ok(command) => match command {
                Command::Version(id) => (),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_known_command() {
        let command: Result<Command, _> = "known_command given_command".parse();

        match command {
            Ok(command) => match command {
                Command::KnownCommand(id, command) => assert_eq!(command, "given_command"),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_list_commands() {
        let command: Result<Command, _> = "list_commands".parse();

        match command {
            Ok(command) => match command {
                Command::ListCommands(id) => (),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }
    
    #[test]
    pub fn test_quit() {
        let command: Result<Command, _> = "quit".parse();

        match command {
            Ok(command) => match command {
                Command::Quit(id) => (),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_board_size() {
        let command: Result<Command, _> = "boardsize 19".parse();

        match command {
            Ok(command) => match command {
                Command::BoardSize(id, size) => assert_eq!(size, 19),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_clear_board() {
        let command: Result<Command, _> = "clear_board".parse();

        match command {
            Ok(command) => match command {
                Command::ClearBoard(id) => (),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_Komi() {
        let command: Result<Command, _> = "komi 6.5".parse();

        match command {
            Ok(command) => match command {
                Command::Komi(id, komi) => assert_eq!(6.5, komi),
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_play() {
        let command: Result<Command, _> = "play B k10".parse();

        match command {
            Ok(command) => match command {
                Command::Play(id, moving) => {
                    match moving.color {
                        Color::Black => (),
                        Color::White => assert!(false, "Wrong color"),
                    };

                    match moving.vertex {
                        Vertex::Pass => assert!(false, "It pass but it shouldn't be"),
                        Vertex::Coordinate(letter, number) => {
                            assert_eq!(letter, 'k');
                            assert_eq!(number, 10);
                        },
                    };
                },
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }

    #[test]
    pub fn test_gen_move() {
        let command: Result<Command, _> = "genmove white".parse();

        match command {
            Ok(command) => match command {
                Command::GenMove(id, color) => {
                    match color {
                        Color::White => (),
                        Color::Black => assert!(false, "The color is not correct"),
                    };
                },
                _ => assert!(false, "Incorrect command"),
            },
            Err(_) => assert!(false, "Parse command Error"),
        };
    }
}
