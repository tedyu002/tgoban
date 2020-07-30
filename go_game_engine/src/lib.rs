mod link;

use std::sync::Arc;

use go_board::{GoBoard, ChessChange, MoveError, Location, ChessType};
use crate::link::Tree;

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
        let grow_node = {
            let head_node = &self.tree.head.borrow().data;

            let chess_type = match head_node.player {
                None => ChessType::Black,
                Some(player) => match player {
                    Player::Black => ChessType::White,
                    Player::White => ChessType::Black,
                },
            };

            match self.board.make_move(chess_type, location) {
                Ok(chess_change) => {
                    let mut node = GoNode {
                        changes: Some(chess_change),
                        steps: head_node.steps + 1,
                        deads: head_node.deads.clone(),
                        player: match head_node.player {
                            None => Some(Player::Black),
                            Some(player) => Some(player.switch()),
                        }
                    };

                    node.deads[node.player.unwrap() as usize] += node.changes.as_ref().unwrap().remove.len() as i32;
                    node
                },
                Err(err) => {
                    return Err(err);
                },
            }
        };

        self.tree.grow(grow_node);

        return Ok(());
    }

    pub fn get_chess(&self, location: Location) -> ChessType {
        return self.board.get(&location);
    }

    pub fn pass(&mut self) {
        let node = {
            let head_node = &self.tree.head.borrow().data;

            Some(GoNode {
                changes: None,
                steps: head_node.steps + 1,
                deads: head_node.deads.clone(),
                player: match head_node.player {
                    None => Some(Player::Black),
                    Some(player) => Some(player.switch()),
                }
            })
        };

        /* Maybe potenstion HEAD is not the same */
        if let Some(node) = node {
            self.tree.grow(node);
        }
    }

    pub fn player(&self) -> Player {
        match &self.tree.head.borrow().data.player {
            None => Player::Black,
            Some(player) => player.switch(),
        }
    }

    pub fn deads(&self, player: &Player) -> i32 {
        return self.tree.head.borrow().data.deads[*player as usize];
    }

    pub fn regret(&mut self) {
        let parent = Some({
            match &self.tree.head.borrow().parent {
                None => {return;},
                Some(weak_node) => {
                    /* Should always success since the root can walk to it */
                    let parent = weak_node.upgrade().unwrap();
                    {
                        let mut parent = parent.borrow_mut();

                        let index = {
                            parent.children.iter().position(|x| {
                                Arc::ptr_eq(&x, &self.tree.head)
                            }).unwrap()
                        };
                        parent.children.remove(index);
                    }

                    parent
                }
            }
        });

        if let Some(parent) = parent {
            match &self.tree.head.borrow().data.changes {
                Some(chess_change) => {
                    self.board.reverse_change(chess_change);
                },
                None => {},
            }
            self.tree.head = parent;
        }
    }
}

impl std::fmt::Display for GoGameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let head_node = &self.tree.head.borrow().data;

        if let Err(err) = write!(f, "\tSteps: {}\n", head_node.steps + 1) {
            return Err(err);
        };

        let player = match head_node.player {
            None => Player::Black,
            Some(player) => player.switch(),
        };

        if let Err(err) = write!(f, "\tPlayer : {}\n", match player {Player::Black => 'X', Player::White => 'O'}) {
            return Err(err);
        }

        if let Err(err) = write!(f, "{}", self.board) {
            return Err(err);
        }

        return Ok(());
    }
}
