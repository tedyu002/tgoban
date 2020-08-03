mod tree;
mod board;

pub use crate::board::{GoBoard, ChessChange, MoveError, Location, ChessType};
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

pub struct GoNode {
    changes: Option<ChessChange>,
    steps: i32,
    deads: [i32; PLAYER_NUM],
    player: Option<Player>,
}

pub struct GoGameEngine {
    board: GoBoard,
    tree: Tree<GoNode>,
}

impl GoGameEngine {
    pub fn new(size: u8) -> GoGameEngine {
        let root_node = GoNode {
            changes: None,
            steps: 0,
            deads: [0; PLAYER_NUM],
            player: None,
        };
        GoGameEngine {
            tree: Tree::new(root_node),
            board: GoBoard::new(size),
        }
    }

    pub fn size(&self) -> u8 {
        return self.board.size();
    }

    pub fn make_move(&mut self, location: Location) -> Result<(), MoveError> {
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
                        deads: head.deads.clone(),
                        player: match head.player {
                            None => Some(Player::Black),
                            Some(player) => Some(player.switch()),
                        },
                    };

                    node.deads[node.player.unwrap() as usize] += node.changes.as_ref().unwrap().remove.len() as i32;

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

    pub fn pass(&mut self) {
        self.tree.grow(|head_data| {
            GoNode {
                changes: None,
                steps: head_data.steps + 1,
                deads: head_data.deads.clone(),
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

    pub fn deads(&self, player: &Player) -> i32 {
        let mut deads:i32 = 0;

        self.tree.access_head(|head| {
            deads = head.deads[*player as usize];
        });

        return deads;
    }

    pub fn regret(&mut self) {
        let mut chess_change: Option<ChessChange> = None;

        self.tree.remove_head(|node_data| {
            chess_change = node_data.changes.clone();
        });

        if let Some(chess_change) = chess_change {
            self.board.reverse_change(&chess_change);
        }
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
