use std::num::{ParseIntError, ParseFloatError};
use std::str::{ParseBoolError};

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

pub type Id = i32;
pub type Integer = i32;
pub type Float = f32;

pub enum Command {
    ProtocolVersion(Option<Id>),
    Name(Option<Id>),
    Version(Option<Id>),
    KnownCommand(Option<Id>, String),
    ListCommands(Option<Id>),
    Quit(Option<Id>),
    BoardSize(Option<Id>, Integer),
    ClearBoard(Option<Id>),
    Komi(Option<Id>, Float),
    Play(Option<Id>, Move),
    GenMove(Option<Id>, Color),
}

impl std::str::FromStr for Command {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut tokens: Vec<&str> = command.split_ascii_whitespace().collect();

        if tokens.len() == 0 {
            return Err(());
        }

        let id: Result<Id, ParseIntError> = tokens[0].parse();

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
                let boardsize: Result<Integer, ParseIntError> = tokens[0].parse();

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

                let new_komi: Result<Float, ParseFloatError> = tokens[0].parse();
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

impl std::string::ToString for Command {
    fn to_string(&self) -> String {
        match self {
            Self::ProtocolVersion(id) => {
                match id {
                    Some(id) => {
                        format!("{} {}", id, "protocol_version")
                    },
                    None => {
                        "protocol_version".to_string()
                    },
                }
            },
            Self::Name(id) => {
                match id {
                    Some(id) => {
                        format!("{} {}", id, "name")
                    },
                    None => {
                        "name".to_string()
                    },
                }
            },
            Self::Version(id) => {
                match id {
                    Some(id) => {
                        format!("{} {}", id, "version")
                    },
                    None => {
                        "version".to_string()
                    },
                }
            },
            Self::KnownCommand(id, command) => {
                match id {
                    Some(id) => {
                        format!("{} {} {}", id, "known_command", command)
                    },
                    None => {
                        format!("{} {}", "known_command", command)
                    },
                }
            },
            Self::ListCommands(id) => {
                match id {
                    Some(id) => {
                        format!("{} {}", id, "list_commands")
                    },
                    None => {
                        "list_commands".to_string()
                    },
                }
            },
            Self::Quit(id) => {
                match id {
                    Some(id) => {
                        format!("{} {}", id, "quit")
                    },
                    None => {
                        "quit".to_string()
                    },
                }
            },
            Self::BoardSize(id, size) => {
                match id {
                    Some(id) => {
                        format!("{} {} {}", id, "boardsize", size)
                    },
                    None => {
                        format!("{} {}", "boardsize", size)
                    },
                }
            },
            Self::ClearBoard(id) => {
                match id {
                    Some(id) => {
                        format!("{} {}", id, "clear_board")
                    },
                    None => {
                        "clear_board".to_string()
                    },
                }
            },
            Self::Komi(id, komi) => {
                match id {
                    Some(id) => {
                        format!("{} {} {}", id, "komi", komi)
                    },
                    None => {
                        format!("{} {}", "komi", komi)
                    },
                }
            },
            Self::Play(id, mov) => {
                match id {
                    Some(id) => {
                        format!("{} {} {}", id, "play", mov.to_string())
                    },
                    None => {
                        format!("{} {}", "play", mov.to_string())
                    },
                }
            },
            Self::GenMove(id, color) => {
                match id {
                    Some(id) => {
                        format!("{} {} {}", id, "genmove", color.to_string())
                    },
                    None => {
                        format!("{} {}", "genmove", color.to_string())
                    },
                }
            },
        }
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

impl std::string::ToString for Color {
    fn to_string(&self) -> String {
        match self {
            Color::White => "w".to_string(),
            Color::Black => "b".to_string(),
        }
    }
}

pub enum Vertex {
    Pass,
    Coordinate(char, Integer),
}

impl Vertex {
    pub fn to_number(&self) -> Result<(u8, u8), ()> {
        match self {
            Vertex::Pass => Err(()),
            Vertex::Coordinate(letter, number) => {
                let mut alphabet = match letter.is_ascii_uppercase() {
                    true => *letter as usize - 'A' as usize,
                    false => *letter as usize - 'a' as usize,
                };
                if alphabet >= 'I' as usize - 'A' as usize {
                    alphabet -= 1;
                }

                Ok((alphabet as u8, (number - 1) as u8))
            },
        }
    }

    pub fn from_number(alphabet: u8, digit: u8) -> Vertex {
        Vertex::Coordinate(
            ('A' as u8 + alphabet + match alphabet >= ('I' as u8 - 'A' as u8) {
                true => 1,
                false => 0,
            }) as char,
            digit as i32 + 1,
        )
    }
}

impl std::string::ToString for Vertex {
    fn to_string(&self) -> String {
        match self {
            Vertex::Pass => {
                "pass".to_string()
            },
            Vertex::Coordinate(letter, number) => {
                format!("{}{}", letter, number)
            },
        }
    }
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

        let number: Integer = match number.parse() {
            Ok(num) => num,
            Err(_) => {
                return Err(());
            }
        };

        return Ok(Vertex::Coordinate(letter, number));
    }
}

pub struct Move {
    pub color: Color,
    pub vertex: Vertex,
}

impl std::string::ToString for Move {
    fn to_string(&self) -> String {
        format!("{} {}", self.color.to_string(), self.vertex.to_string())
    }
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

pub enum GenMoveResult {
    Resign,
    Move(Vertex),
}

impl std::str::FromStr for GenMoveResult {
    type Err = ();
    fn from_str(command: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        if command == "resign" {
           return Ok(Self::Resign);
        }

        let vertex: Result<Vertex, ()> = command.parse();

        match vertex {
            Err(_) => {
                return Err(());
            },
            Ok(vertex) => {
                Ok(Self::Move(vertex))
            }
        }
    }
}

impl std::string::ToString for GenMoveResult {
    fn to_string(&self) -> String {
        match self {
            GenMoveResult::Resign => "resign".to_string(),
            GenMoveResult::Move(vertex) => {
                vertex.to_string()
            },
        }
    }
}

pub enum CommandResult {
    ProtocolVersion(Option<Id>, Integer),
    Name(Option<Id>, String),
    Version(Option<Id>, String),
    KnownCommand(Option<Id>, bool),
    ListCommands(Option<Id>, Vec<String>),
    Quit(Option<Id>),
    BoardSize(Option<Id>),
    ClearBoard(Option<Id>),
    Komi(Option<Id>),
    Play(Option<Id>),
    GenMove(Option<Id>, GenMoveResult)
}

impl Command {
    pub fn parse_result(&self, lines: &str) -> Result<CommandResult, ()> {
        let mut lines = lines.lines();

        let first_line = lines.next();

        if let None = first_line {
            return Err(());
        }

        /* Start to get id */
        let first_line = first_line.unwrap();
        let space_idx = match first_line.find(' ') {
            Some(idx) => idx,
            None => return Err(()),
        };

        let id = {
            let header = &first_line[0..space_idx];

            match header.find('=') {
                Some(idx) => match idx {
                    0 => {},
                    _ => return Err(()),
                },
                None => return Err(()),
            };

            let header = &first_line[1..space_idx];

            match header.len() {
                0 => None,
                _ => {
                    let id: Result<Integer, ParseIntError> = header.parse();
                    match id {
                        Err(_) => return Err(()),
                        Ok(number) => Some(number),
                    }
                }
            }
        };

        let lines: Vec<String> = (first_line[(space_idx+1)..]).lines().chain(lines)
            .map(|x| x.trim().to_string())
            .filter(|x| x.len() > 0)
            .collect();

        let result = match self {
            Self::ProtocolVersion(_id) => {
                if lines.len() != 1 {
                    return Err(());
                }
                let version: Result<Integer, ParseIntError> = lines[0].parse();
                if let Err(_) = version {
                    return Err(());
                }

                CommandResult::ProtocolVersion(id, version.unwrap())
            },
            Self::Name(_id) => {
                if lines.len() != 1 {
                    return Err(());
                }

                CommandResult::Name(id, lines[0].to_string())
            },
            Self::Version(_id) => {
                if lines.len() != 1 {
                    return Err(());
                }
                CommandResult::Version(id, lines[0].to_string())
            },
            Self::KnownCommand(_id, _command) => {
                if lines.len() != 1 {
                    return Err(());
                }

                let result: Result<bool, ParseBoolError> = lines[0].parse();

                match result {
                    Err(_) => return Err(()),
                    Ok(known) => CommandResult::KnownCommand(id, known),
                }
            },
            Self::ListCommands(_id) => {
                CommandResult::ListCommands(id, lines)
            },
            Self::Quit(_id) => {
                if lines.len() != 1 {
                    return Err(());
                }
                CommandResult::Quit(id)
            },
            Self::BoardSize(_id, _size) => {
                if lines.len() != 1 {
                    return Err(());
                }
                CommandResult::BoardSize(id)
            },
            Self::ClearBoard(_id) => {
                if lines.len() != 1 {
                    return Err(());
                }
                CommandResult::ClearBoard(id)
            },
            Self::Komi(_id, _komi) => {
                if lines.len() != 1 {
                    return Err(());
                }
                CommandResult::Komi(id)
            },
            Self::Play(_id, _mov) => {
                if lines.len() != 1 {
                    return Err(());
                }
                CommandResult::Play(id)
            },
            Self::GenMove(_id, _color) => {
                if lines.len() != 1 {
                    return Err(());
                }
                let result: Result<GenMoveResult, ()> = lines[0].parse();

                match result {
                    Err(_) => return Err(()),
                    Ok(mov) => CommandResult::GenMove(id, mov),
                }
            },
        };

        Ok(result)
    }
}

impl std::string::ToString for CommandResult {
    fn to_string(&self) -> String {
        match self {
            Self::ProtocolVersion(id, version) => {
                match id {
                    Some(id) => {
                        format!("={} {}", id, version)
                    },
                    None => {
                        format!("= {}", version)
                    },
                }
            },
            Self::Name(id, name) => {
                match id {
                    Some(id) => {
                        format!("={} {}", id, name)
                    },
                    None => {
                        format!("= {}", name)
                    },
                }
            },
            Self::Version(id, version) => {
                match id {
                    Some(id) => {
                        format!("={} {}", id, version)
                    },
                    None => {
                        format!("= {}", version)
                    },
                }
            },
            Self::KnownCommand(id, is_known) => {
                match id {
                    Some(id) => {
                        format!("={} {}", id, match is_known {
                            true => "true",
                            false => "false",
                        })
                    },
                    None => {
                        format!("= {}", match is_known {
                            true => "true",
                            false => "false",
                        })
                    },
                }
            },
            Self::ListCommands(id, commands) => {
                let mut reply = match id {
                    Some(id) => {
                        format!("={} ", id)
                    },
                    None => {
                        "= ".to_string()
                    },
                };

                let it = commands.iter().map(|x| format!("{} \n", x));

                for s in it {
                    reply.push_str(&s);
                }

                reply
            },
            Self::Quit(id) => {
                match id {
                    Some(id) => {
                        format!("={} ", id)
                    },
                    None => {
                        "= ".to_string()
                    },
                }
            },
            Self::BoardSize(id) => {
                match id {
                    Some(id) => {
                        format!("={} ", id)
                    },
                    None => {
                        "= ".to_string()
                    },
                }
            },
            Self::ClearBoard(id) => {
                match id {
                    Some(id) => {
                        format!("={} ", id)
                    },
                    None => {
                        "= ".to_string()
                    },
                }
            },
            Self::Komi(id) => {
                match id {
                    Some(id) => {
                        format!("={} ", id)
                    },
                    None => {
                        "= ".to_string()
                    },
                }
            },
            Self::Play(id) => {
                match id {
                    Some(id) => {
                        format!("={} ", id)
                    },
                    None => {
                        "= ".to_string()
                    },
                }
            },
            Self::GenMove(id, move_result) => {
                match id {
                    Some(id) => {
                        format!("={} {}", id, move_result.to_string())
                    },
                    None => {
                        format!("= {}", move_result.to_string())
                    },
                }
            },
        }
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
