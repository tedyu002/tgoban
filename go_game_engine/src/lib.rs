mod tree;
mod board;
mod scoring_board;

pub use crate::board::{GoBoard, ChessChange, MoveError, Location, ChessType};
use crate::scoring_board::ScoreBoard;
use crate::tree::{Tree};


const PLAYER_NUM: usize = 2;

#[derive(Copy, Clone)]
pub enum Player {
    Black = 0,
    White = 1,
}

impl Player {
    pub fn switch(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameStatus {
    Playing,
    Scoring,
}

pub struct GoNode {
    changes: Option<ChessChange>,
    steps: i32,
    capture: [i32; PLAYER_NUM],
    player: Option<Player>,
}

pub struct GoGameEngine {
    board: GoBoard,
    tree: Tree<GoNode>,
    status: GameStatus,
    score_board: Option<ScoreBoard>,
}

impl GoGameEngine {
    pub fn new(size: u8) -> GoGameEngine {
        let root_node = GoNode {
            changes: None,
            steps: 0,
            capture: [0; PLAYER_NUM],
            player: None,
        };
        GoGameEngine {
            tree: Tree::new(root_node),
            board: GoBoard::new(size),
            status: GameStatus::Playing,
            score_board: None,
        }
    }

    pub fn size(&self) -> u8 {
        return self.board.size();
    }

    pub fn make_move(&mut self, location: Location) -> Result<(), MoveError> {
        match self.status {
            GameStatus::Playing => {},
            _ => return Ok(()),
        };

        let mut chess_type = ChessType::None;

        self.tree.access_head(|head| {
           chess_type = match head.player {
                None => ChessType::Black,
                Some(player) => match player {
                    Player::Black => ChessType::White,
                    Player::White => ChessType::Black,
                },
            };
        });

        match self.board.make_move(chess_type, location) {
            Ok(chess_change) => {
                self.tree.grow(|head| {
                    let mut node = GoNode {
                        changes: Some(chess_change),
                        steps: head.steps + 1,
                        capture: head.capture.clone(),
                        player: match head.player {
                            None => Some(Player::Black),
                            Some(player) => Some(player.switch()),
                        },
                    };

                    node.capture[node.player.unwrap() as usize] += node.changes.as_ref().unwrap().remove.len() as i32;

                    return node;
                });
            },
            Err(err) => {
                return Err(err);
            },
        }

        return Ok(());
    }

    pub fn get_chess(&self, location: Location) -> ChessType {
        return self.board.get(&location);
    }

    pub fn get_belong(&self, location: Location) -> Option<Player> {
        match self.status {
            GameStatus::Scoring => {},
            _ => return None,
        };

        return self.score_board.as_ref().unwrap().get_belong(location);
    }

    pub fn pass(&mut self) {
        match self.status {
            GameStatus::Playing => {},
            _ => return,
        };

        let mut status: Option<GameStatus> = None;
        self.tree.access_head(|head| {
            if let None = head.changes {
                status = Some(GameStatus::Scoring);
            }
        });

        match status {
            None => {},
            Some(game_status) => {
                self.status = game_status;
                self.score_board = Some(ScoreBoard::new(&self.board));
                self.score_board.as_mut().unwrap().refresh_belong(&self.board);
                return;
            }
        };

        self.tree.grow(|head_data| {
            GoNode {
                changes: None,
                steps: head_data.steps + 1,
                capture: head_data.capture.clone(),
                player: match head_data.player {
                    None => Some(Player::Black),
                    Some(player) => Some(player.switch()),
                }
            }
        });
    }

    pub fn player(&self) -> Player {
        let mut player = Player::Black;

        self.tree.access_head(|head| {
            player = match head.player {
                None => Player::Black,
                Some(player) => player.switch(),
            };
        });

        return player;
    }

    pub fn get_capture(&self, player: &Player) -> i32 {
        let mut capture:i32 = 0;

        self.tree.access_head(|head| {
            capture = head.capture[*player as usize];
        });

        if self.status == GameStatus::Scoring {
            capture += self.score_board.as_ref().unwrap().get_capture(&self.board, player);
        }

        return capture;
    }

    pub fn steps(&self) -> i32 {
        let mut steps: i32 = 0;
        self.tree.access_head(|head| {
            steps = head.steps;
        });

        return steps;
    }

    pub fn regret(&mut self) {
        self.status = GameStatus::Playing;

        let mut chess_change: Option<ChessChange> = None;

        self.tree.remove_head(|node_data| {
            chess_change = node_data.changes.clone();
        });

        if let Some(chess_change) = chess_change {
            self.board.reverse_change(&chess_change);
        }
    }

    pub fn get_status(&self) -> GameStatus {
        return self.status;
    }

    pub fn toggle(&mut self, location: Location) {
        if self.status != GameStatus::Scoring {
            return;
        }

        match self.board.get(&location) {
            ChessType::None => return,
            ChessType::Black | ChessType::White => {
                let score_board = self.score_board.as_mut().unwrap();

                score_board.toggle(&self.board, location);
                score_board.refresh_belong(&self.board);
            }
        };
    }

    pub fn is_alive(&self, location: Location) -> bool {
        if self.status != GameStatus::Scoring {
            return false;
        }
        return self.score_board.as_ref().unwrap().is_alive(location);
    }
}

impl std::fmt::Display for GoGameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res: std::fmt::Result = Ok(());

        self.tree.access_head(|head| {
            if let Err(err) = write!(f, "\tSteps: {}\n", head.steps + 1) {
                res = Err(err);
                return;
            };

            let player = match head.player {
                None => Player::Black,
                Some(player) => player.switch(),
            };

            if let Err(err) = write!(f, "\tPlayer : {}\n", match player {Player::Black => 'X', Player::White => 'O'}) {
                res = Err(err);
                return;
            }

            if let Err(err) = write!(f, "{}", self.board) {
                res = Err(err);
                return;
            }
        });

        return res;
    }
}
