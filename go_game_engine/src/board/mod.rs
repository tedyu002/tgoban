pub(crate) mod go_board;
pub(crate) mod scoring_board;

use crate::Location;
use crate::BOARD_SIZE_MAX;

pub struct Board<T: Copy> {
    pub size: u8,
    pub board: [[T; BOARD_SIZE_MAX]; BOARD_SIZE_MAX],
}

impl<T: Copy> Board<T> {
    pub fn get(&self, location: &Location) -> T {
        return self.board[location.alphabet as usize][location.digit as usize];
    }

    pub fn size(&self) -> u8 {
        return self.size;
    }

    pub fn set(&mut self, location: &Location, t: T) {
        self.board[location.alphabet as usize][location.digit as usize] = t;
    }

    pub fn neighbors(&self, location: &Location) -> Vec<Location>{
        let mut neighbors: Vec<Location> = Vec::new();

        if location.alphabet > 0 {
            neighbors.push(Location {
                alphabet: location.alphabet - 1,
                digit: location.digit,
            });
        }

        if location.alphabet < self.size - 1 {
            neighbors.push(Location {
                alphabet: location.alphabet + 1,
                digit: location.digit,
            });
        }

        if location.digit > 0 {
            neighbors.push(Location {
                alphabet: location.alphabet ,
                digit: location.digit - 1,
            });
        }

        if location.digit < self.size - 1 {
            neighbors.push(Location {
                alphabet: location.alphabet,
                digit: location.digit + 1,
            });
        }

        return neighbors;
    }
}
