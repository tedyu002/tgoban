use crate::{Player, Location};
use crate::BOARD_SIZE_MAX;
use crate::board::go_board::{GoBoard, ChessType};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Live {
    Dead,
    Alive,
    None,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Belong {
    Black,
    White,
    None,
}

type LiveBoard = crate::board::Board<Live>;
impl LiveBoard {
    fn new(size: u8) -> LiveBoard {
        LiveBoard {
            size,
            board: [[Live::None; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
        }
    }
}

type BelongBoard = crate::board::Board<Belong>;
impl BelongBoard {
    fn new (size: u8) -> BelongBoard {
        BelongBoard {
            size,
            board: [[Belong::None; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
        }
    }
}

pub(crate) struct ScoreBoard {
    size: u8,
    live_board: LiveBoard,
    belong_board: BelongBoard,
}

impl ScoreBoard {
    pub fn new(go_board: &GoBoard) -> ScoreBoard {
        let mut score_board = ScoreBoard {
            size: go_board.size,
            belong_board: BelongBoard::new(go_board.size),
            live_board: LiveBoard::new(go_board.size),
        };

        for alphabet in 0..score_board.size {
            for digit in 0..score_board.size {
                let location = Location {
                    alphabet,
                    digit,
                };

                let live_status = match go_board.get(&location) {
                    ChessType::Black | ChessType::White => {
                        Live::Alive
                    },
                    ChessType::None => {
                        continue;
                    },
                };

                score_board.live_board.set(&location, live_status);
            }
        }

        score_board
    }

    pub fn get_belong(&self, location: Location) -> Option<Player> {
        match self.belong_board.get(&location) {
            Belong::Black => Some(Player::Black),
            Belong::White => Some(Player::White),
            _ => None,
        }
    }

    pub fn refresh_belong(&mut self, go_board: &GoBoard) {
        let mut white_belong_board = BelongBoard::new(self.size);
        let mut black_belong_board = BelongBoard::new(self.size);

        let mark = | go_board: &GoBoard, belong_board: &mut BelongBoard, live_board: &LiveBoard, chess_type | {
            for alphabet in 0..go_board.size {
                for digit in 0..go_board.size {
                    let location = Location {
                        alphabet,
                        digit,
                    };
                    let board_chess= go_board.get(&location);
                    let live = live_board.get(&location);

                    if board_chess == chess_type && live == Live::Alive {
                        let mut check_queue: Vec<Location> = Vec::new();

                        for neighbor in go_board.neighbors(&location) {
                            check_queue.push(neighbor);
                        }

                        while let Some(location) = check_queue.pop() {
                            if belong_board.get(&location) != Belong::None {
                                continue;
                            }
                            if live_board.get(&location) != Live::Alive {
                                belong_board.set(&location, match chess_type {
                                    ChessType::Black => Belong::Black,
                                    ChessType::White => Belong::White,
                                    ChessType::None => panic!("The chess type cannot be None"),
                                });

                                for neighbor in go_board.neighbors(&location).iter() {
                                    check_queue.push(*neighbor);
                                }
                            }
                        }
                    }
                }
            }
        };

        mark(go_board, &mut white_belong_board, &self.live_board, ChessType::White);
        mark(go_board, &mut black_belong_board, &self.live_board, ChessType::Black);

        for alphabet in 0..self.size {
            for digit in 0..self.size {
                let location = Location {
                    alphabet,
                    digit,
                };
                let is_belong_white = white_belong_board.get(&location) != Belong::None;
                let is_belong_black = black_belong_board.get(&location) != Belong::None;

                if is_belong_white && is_belong_black {
                    self.belong_board.set(&location, Belong::None);
                } else if is_belong_white {
                    self.belong_board.set(&location, Belong::White);
                } else if is_belong_black {
                    self.belong_board.set(&location, Belong::Black);
                } else {
                    self.belong_board.set(&location, Belong::None);
                }
            }
        }
    }

    pub fn toggle(&mut self, go_board: &GoBoard, location: Location) {
        let mut queue: Vec<Location> = Vec::new();
        queue.push(location);

        let new_status = match self.live_board.get(&location) {
            Live::Alive => Live::Dead,
            Live::Dead => Live::Alive,
            Live::None => return,
        };

        let chess = go_board.get(&location);

        while let Some(location) = queue.pop() {
            self.live_board.set(&location, new_status);

            for neighbor in go_board.neighbors(&location) {
                if go_board.get(&neighbor) != chess {
                    continue;
                }

                if self.live_board.get(&neighbor) == new_status {
                    continue;
                }
                queue.push(neighbor);
            }
        }
    }

    pub fn is_alive(&self, location: Location) -> bool{
        match self.live_board.get(&location) {
            Live::Alive => true,
            _ => false,
        }
    }

    pub fn get_capture(&self, go_board: &GoBoard, player: &Player) -> i32 {
        let mut capture: (i32, i32) = (0, 0);
        for alphabet in 0..self.size {
            for digit in 0..self.size {
                let location = Location {
                    alphabet,
                    digit,
                };
                if self.live_board.get(&location) == Live::Dead {
                    let board_chess = go_board.get(&location);
                    match board_chess {
                        ChessType::None => panic!("The dead could no be no chess"),
                        ChessType::Black => {
                            capture.1 += 1;
                        },
                        ChessType::White => {
                            capture.0 += 1;
                        }
                    };
                }
            }
        }

        match player {
            Player::Black => capture.0,
            Player::White => capture.1,
        }
    }
}
