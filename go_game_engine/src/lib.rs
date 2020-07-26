use go_board::{GoBoard, ChessChange, MoveError, Location, ChessType, Chess};

#[derive(Copy, Clone)]
pub enum Player {
    Black,
    White,
}

pub struct GoGameEngine {
    board: GoBoard,
    steps: Vec<ChessChange>,
    player: Player,
}

impl GoGameEngine {
    pub fn new(size: u8) -> GoGameEngine {
        GoGameEngine {
            board: GoBoard::new(size),
            steps: Vec::new(),
            player: Player::Black,
        }
    }

    pub fn size(&self) -> u8 {
        return self.board.size();
    }

    pub fn make_move(&mut self, location: Location) -> Result<(), MoveError> {
        let chess_type = match self.player {
            Player::Black => ChessType::Black,
            Player::White => ChessType::White,
        };

        match self.board.make_move(chess_type, location) {
            Ok(chess_change) => {
                self.steps.push(chess_change);
                self.switch_player();
            },
            Err(err) => {
                return Err(err);
            },
        };

        return Ok(());
    }

    pub fn get_chess(&self, location: Location) -> ChessType {
        return self.board.get(&location);
    }

    pub fn pass(&mut self) {
        self.steps.push(ChessChange {
            at: Chess {
                chess_type: ChessType::None,
                location: Location {
                    x: 0,
                    y: 0,
                }
            },
            remove: Vec::new(),
        });
        self.switch_player();
    }

    fn switch_player(&mut self) {
        self.player = match self.player {
            Player::Black => Player::White,
            Player::White => Player::Black,
        };
    }
}

impl std::fmt::Display for GoGameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Err(err) = write!(f, "\tSteps: {}\n", self.steps.len() + 1) {
            return Err(err);
        };

        if let Err(err) = write!(f, "\tPlayer : {}\n", match self.player {Player::Black => 'X', Player::White => 'O'}) {
            return Err(err);
        }

        if let Err(err) = write!(f, "{}", self.board) {
            return Err(err);
        }

        return Ok(());
    }
}
