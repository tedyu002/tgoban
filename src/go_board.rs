use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::num::ParseIntError;

const BOARD_SIZE_MAX: usize = 19;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChessType {
    None = 0,
    Black = 1,
    White = 2,
}


#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub x: u8,
    pub y: u8,
}

impl Location {
    pub fn new() -> Location {
        Location {
            x: 0,
            y: 0,
        }
    }

    fn set(&mut self, idx: u8, val:u8) {
        if idx == 0 {
            self.x = val;
        } else if idx == 1 {
            self.y = val;
        } else {
            panic!("Out of range");
        }
    }
}

pub struct ParseTwoIntError {
}

impl std::str::FromStr for Location {
    type Err = ParseTwoIntError;

    fn from_str(line: &str) -> Result<Location, ParseTwoIntError> {
        let fields: Vec<&str> = line.split(" ").filter(|s| s.len() > 0).collect();

        if fields.len() != 2 {
            return Err(ParseTwoIntError {
            })
        } else {
            let mut location = Location::new();

            for idx in 0..2 {
                match fields[idx].parse::<isize>() {
                    Ok(num) => {
                        location.set(idx as u8, num as u8);
                    },
                    Err(_) => {
                        return Err(ParseTwoIntError {});
                    }
                };
            }

            return Ok(location)
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Chess {
    pub chess_type: ChessType,
    pub location: Location,
}
pub struct ChessChange {
    pub at: Chess,
    pub remove: BTreeSet<Chess>,
}

pub enum MoveError {
    Exist(Location),
    NoLiberty(Location),
    NoMove,
}

pub struct GoBoard {
    size: u8,
    board: [[ChessType; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
}

impl GoBoard {
    pub fn new(size: u8) -> GoBoard {
        GoBoard {
            size,
            board: [[ChessType::None; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
        }
    }

    pub fn make_move(&mut self, chess_type: ChessType, location: Location) -> Result<ChessChange, MoveError> {
        let board_chess = self.board[location.x as usize][location.y as usize];

        match board_chess {
            ChessType::None => {
                self.board[location.x as usize][location.y as usize] = chess_type;
                let deads = GoBoardLiberty::get_deads(self);

                match chess_type {
                    ChessType::Black => {
                        if deads.0.len() > 0 && deads.1.len() == 0 {
                            self.board[location.x as usize][location.y as usize] = ChessType::None;
                            return Err(MoveError::NoLiberty(location));
                        }
                    },
                    ChessType::White => {
                        if deads.1.len() > 0 && deads.0.len() == 0 {
                            self.board[location.x as usize][location.y as usize] = ChessType::None;
                            return Err(MoveError::NoLiberty(location));
                        }
                    },
                    ChessType::None => {
                        return Err(MoveError::NoMove);
                    }
                };

                Ok(ChessChange {
                    at: Chess {
                        chess_type,
                        location,
                    },
                    remove: BTreeSet::new()
                })
            },
            _ => {
                Err(MoveError::Exist(location))
            }
        }
    }

}

struct GoBoardLiberty {
    size: u8,
    board: [[bool; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
}

impl GoBoardLiberty {
    fn new(size: u8) -> GoBoardLiberty {
        GoBoardLiberty {
            size,
            board: [[false; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
        }
    }

    fn get_deads(board: &GoBoard) -> (Vec<(i16, i16)>, Vec<(i16, i16)>) {
        let mut deads: (Vec<(i16, i16)>, Vec<(i16, i16)>) = (Vec::new(), Vec::new());

        let board_liberty = GoBoardLiberty::make(board);

        for idx1 in 0..board.size {
            for idx2 in 0..board.size {
                match board.board[idx1 as usize][idx2 as usize] {
                    ChessType::Black => {
                        match board_liberty.board[idx1 as usize][idx2 as usize] {
                            true => continue,
                            false => {
                                deads.0.push((idx1 as i16, idx2 as i16));
                            }
                        }
                    },
                    ChessType::White => {
                        match board_liberty.board[idx1 as usize][idx2 as usize] {
                            true => continue,
                            false => {
                                deads.1.push((idx1 as i16, idx2 as i16));
                            }
                        }
                    },
                    ChessType::None => continue,
                }
            }
        }        

        deads
    }

    fn make(board: &GoBoard) -> GoBoardLiberty {
        let mut board_liberty = GoBoardLiberty::new(board.size);

        for idx1 in 0..board.size {
            for idx2 in 0..board.size {
                if board.board[idx1 as usize][idx2 as usize] == ChessType::None {
                    let directions = [
                        (idx1 as i16 - 1, idx2 as i16),
                        (idx1 as i16 + 1, idx2 as i16),
                        (idx1 as i16, idx2 as i16 - 1),
                        (idx1 as i16, idx2 as i16 + 1),
                    ];

                    for &location in directions.iter() {
                        let mut ls: Vec<(i16, i16)> = Vec::new();

                        ls.push(location);

                        while ls.len() > 0 {
                            let location = ls.pop().unwrap();

                            if location.0 < 0 || location.0 >= board.size as i16 ||
                                location.1 < 0 || location.1 >= board.size as i16 {
                                continue;
                            }

                            match board_liberty.board[location.0 as usize][location.1 as usize] {
                                false => {
                                    board_liberty.board[location.0 as usize][location.1 as usize] = match board.board[location.0 as usize][location.1 as usize] {
                                        ChessType::None => continue,
                                        ChessType::Black => {
                                            true
                                        },
                                        ChessType::White => {
                                            true
                                        },
                                    }
                                },
                                true => continue,
                            };

                            let directions = [
                                (location.0 - 1, location.1),
                                (location.0 + 1, location.1),
                                (location.0, location.1 - 1),
                                (location.0, location.1 + 1),
                            ];

                            for next_location in directions.iter() {
                                if next_location.0 < 0 || next_location.0 >= board.size as i16 ||
                                    next_location.1 < 0 || next_location.1 >= board.size as i16 {
                                    continue;
                                }

                                if board.board[next_location.0 as usize][next_location.1 as usize] != board.board[location.0 as usize][location.1 as usize] {
                                    continue;
                                }

                                ls.push(*next_location);
                            }
                        }
                    }
                }
            }
        }

        board_liberty
    }
}
