use std::cell::RefCell;
use std::sync::Arc;

use go_game_engine::GoGameEngine;

#[derive(Clone)]
pub struct DruidGoGame {
    pub game: Arc<RefCell<GoGameEngine>>,
    pub version: i32,
}

impl druid::Data for DruidGoGame {
    fn same(&self, other: &Self) -> bool {
        return self.version == other.version;
    }
}
