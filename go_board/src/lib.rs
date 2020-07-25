use std::cmp::Ordering;
use std::num::ParseIntError;

const BOARD_SIZE_MAX: usize = 19;

pub struct Board<T: Copy> {
    size: u8,
    board: [[T; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
}

impl<T: Copy> Board<T> {
    pub fn get(&self, location: &Location) -> T {
        return self.board[location.x as usize][location.y as usize];
    }

    fn set(&mut self, location: &Location, t: T) {
        self.board[location.x as usize][location.y as usize] = t;
    }

    fn neighbors(&self, location: &Location) -> Vec<Location>{
        let mut neighbors: Vec<Location> = Vec::new();

        if location.x > 0 {
            neighbors.push(Location {
                x: location.x - 1,
                y: location.y,
            });
        }

        if location.x < self.size - 1 {
            neighbors.push(Location {
                x: location.x + 1,
                y: location.y,
            });
        }

        if location.y > 0 {
            neighbors.push(Location {
                x: location.x ,
                y: location.y - 1,
            });
        }

        if location.y < self.size - 1 {
            neighbors.push(Location {
                x: location.x,
                y: location.y + 1,
            });
        }

        return neighbors;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChessType {
    None,
    Black,
    White,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
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

pub struct Chess {
    pub chess_type: ChessType,
    pub location: Location,
}

pub struct ChessChange {
    pub at: Chess,
    pub remove: Vec<Location>,
}

impl ChessChange {
    fn new() -> ChessChange {
        ChessChange {
            at: Chess {
                chess_type: ChessType::None,
                location: Location {
                    x: 0,
                    y: 0,
                }
            },
            remove: Vec::new(),
        }
    }
}

pub enum MoveError {
    Exist(Location),
    NoLiberty(Location),
    NoMove,
}

pub type GoBoard = Board<ChessType>;

impl GoBoard {
    pub fn new(size: u8) -> GoBoard {
        GoBoard {
            size,
            board: [[ChessType::None; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
        }
    }

    pub fn make_move(&mut self, chess_type: ChessType, location: Location) -> Result<ChessChange, MoveError> {
        let board_chess = self.get(&location);

        match board_chess {
            ChessType::None => {
                self.set(&location, chess_type);
                let deads = GoBoardLiberty::get_deads(self);

                let mut chess_change = ChessChange::new();

                match chess_type {
                    ChessType::None => {
                        return Err(MoveError::NoMove);
                    },
                    ChessType::Black => {
                        if deads.0.len() > 0 && deads.1.len() == 0 {
                            self.set(&location, ChessType::None);
                            return Err(MoveError::NoLiberty(location));
                        }
                        self.set(&location, chess_type);

                        chess_change.at.chess_type = chess_type;
                        chess_change.at.location = location;
                        chess_change.remove = deads.1;
                    },
                    ChessType::White => {
                        if deads.1.len() > 0 && deads.0.len() == 0 {
                            self.set(&location, ChessType::None);
                            return Err(MoveError::NoLiberty(location));
                        }
                        self.set(&location, chess_type);

                        chess_change.at.chess_type = chess_type;
                        chess_change.at.location = location;
                        chess_change.remove = deads.0;
                    },
                };

                for location in chess_change.remove.iter() {
                    self.set(location, ChessType::None);
                }

                return Ok(chess_change);
            },
            _ => {
                Err(MoveError::Exist(location))
            }
        }
    }

}

impl std::fmt::Display for GoBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.size {
            for j in 0..self.size {
                let location = Location {
                    x: j,
                    y: self.size - i - 1,
                };

                let character = match self.get(&location) {
                    ChessType::None => '.',
                    ChessType::Black => 'X',
                    ChessType::White => 'O',
                };

                if let Err(error) = write!(f, "{}", character) {
                    return Err(error);
                }
            }
            if let Err(error) = write!(f, "{}", "\n") {
                return Err(error);
            }
        }

        return Ok(());
    }
}

type GoBoardLiberty = Board<bool>;

impl GoBoardLiberty {
    fn new(size: u8) -> GoBoardLiberty {
        GoBoardLiberty {
            size,
            board: [[false; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
        }
    }

    fn get_deads(board: &GoBoard) -> (Vec<Location>, Vec<Location>) {
        let mut deads: (Vec<Location>, Vec<Location>) = (Vec::new(), Vec::new());

        let board_liberty = GoBoardLiberty::make(board);

        for idx1 in 0..board.size {
            for idx2 in 0..board.size {
                let location = Location {
                    x: idx1,
                    y: idx2,
                };
                match board.get(&location) {
                    ChessType::Black => {
                        match board_liberty.get(&location) {
                            true => continue,
                            false => {
                                deads.0.push(location);
                            }
                        }
                    },
                    ChessType::White => {
                        match board_liberty.get(&location) {
                            true => continue,
                            false => {
                                deads.1.push(location);
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
                let location = Location {
                    x: idx1,
                    y: idx2,
                };
                if board.get(&location) == ChessType::None {
                    let mut spread_start: Vec<Location> = Vec::new();

                    for location in board.neighbors(&location).iter() {
                        match board_liberty.get(location) {
                            false => {
                                let has_liberty = match board.get(&location) {
                                    ChessType::None => continue,
                                    ChessType::Black => {
                                        true
                                    },
                                    ChessType::White => {
                                        true
                                    },
                                };
                                if !board_liberty.get(location) {
                                    board_liberty.set(location, has_liberty);
                                    spread_start.push(*location);
                                }
                            },
                            true => continue,
                        };
                    }

                    while let Some(spread_location) = spread_start.pop() {
                        for next_location in board.neighbors(&spread_location).iter() {
                            if board.get(&next_location) != board.get(&spread_location) {
                                continue;
                            }

                            if !board_liberty.get(next_location) {
                                board_liberty.set(next_location, true);
                                spread_start.push(*next_location);
                            }
                        }
                    }
                }
            }
        }

        board_liberty
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
