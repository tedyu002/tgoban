mod tree;
mod board;

pub use crate::board::go_board::{GoBoard, ChessChange, MoveError, ChessType};
use crate::board::scoring_board::ScoreBoard;
use crate::tree::{Tree};

pub const BOARD_SIZE_MAX: usize = 19;
pub const PLAYER_NUM: usize = 2;

#[derive(Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Location {
    pub alphabet: u8,
    pub digit: u8,
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
    komi: f64,
    status: GameStatus,
    score_board: Option<ScoreBoard>,
    playAs: Option<Player>,
}

impl GoGameEngine {
    pub fn new(size: u8, komi: f64) -> GoGameEngine {
        let root_node = GoNode {
            changes: None,
            steps: 0,
            capture: [0; PLAYER_NUM],
            player: None,
        };
        GoGameEngine {
            tree: Tree::new(root_node),
            board: GoBoard::new(size),
            komi,
            status: GameStatus::Playing,
            score_board: None,
            playAs: None,
        }
    }

    pub fn size(&self) -> u8 {
        return self.board.size();
    }

    pub fn komi(&self) -> f64 {
        return self.komi;
    }

    pub fn setPlayAs(&mut self, player: Player) {
        self.playAs = Some(player);
    }

    pub fn getPlayAs(&self) -> Option<Player> {
        match self.playAs {
            Some(player) => Some(player),
            None => None,
        }
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
            return true;
        }
        return self.score_board.as_ref().unwrap().is_alive(location);
    }

    pub fn get_score(&self) -> (f64, f64) {
        let mut score: (f64, f64) = (0.0, 0.0);

        if self.status != GameStatus::Scoring {
            return score;
        }
        for alphabet in 0..self.size() {
            for digit in 0..self.size() {
                match self.get_belong(Location {
                    alphabet,
                    digit,
                }) {
                    None => continue,
                    Some(player) => {
                        match player {
                            Player::Black => {
                                score.0 += 1.0;
                            },
                            Player::White => {
                                score.1 += 1.0;
                            },
                        }
                    }
                };
            }
        }

        score.0 -= self.get_capture(&Player::White) as f64;
        score.1 -= self.get_capture(&Player::Black) as f64;

        score.1 += self.komi;

        return score;
    }

    pub fn to_sgf(&self) -> String {
        let mut sgf = "".to_string();

        sgf.push_str("(;");
        sgf.push_str("GM[1]FF[4]CA[UTF-8]AP[TGoBan:0.0.1]RU[Japanese]");
        sgf.push_str(&format!("KM[{}]", self.komi));

        let size = self.size();
        let mut is_root = true;

        self.tree.preorder(|data: &GoNode| {
            if !is_root {
                sgf.push_str(
                    &format!(";{}[{}]\n",
                        match data.player.unwrap() {
                            Player::Black => 'B',
                            Player::White => 'W',
                        },
                        match &data.changes {
                            None => "".to_string(),
                            Some(chess_change) => {
                                format!("{}{}",
                                    (chess_change.at.location.alphabet as u8 + 'a' as u8) as char,
                                    ((size - 1 - chess_change.at.location.digit) as u8 + 'a' as u8) as char
                                )
                            }
                        },
                    )
                );
            }
            is_root = false;
        });

        sgf.push_str(")");

        return sgf;
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

impl std::fmt::Debug for GoGameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
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
            if fields.len() != 2 {
                return Err(ParseTwoIntError{});
            }

            let alphabet_idx = match fields[0].parse::<char>() {
                Ok(alphabet) => {
                    match alphabet {
                        'A'..='Z' if alphabet != 'I' => {
                            alphabet as u8 - 'A' as u8
                        },
                        _ => {
                            return Err(ParseTwoIntError {});
                        }
                    }
                },
                Err(_) => {
                    return Err(ParseTwoIntError {});
                }
            };

            let digit_idx = match fields[1].parse::<u8>() {
                Ok(digit) => {
                    digit - 1
                },
                Err(_) => {
                    return Err(ParseTwoIntError {});
                }
            };

            Ok(Location {
                alphabet: alphabet_idx,
                digit: digit_idx,
            })
        }
    }
}
