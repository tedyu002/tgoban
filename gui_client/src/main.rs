mod board;
mod game;

use std::sync::Arc;
use std::cell::RefCell;

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use go_game_engine::GoGameEngine;

use game::DruidGoGame;
use board::BoardWidget;


const BOARD_SIZE: u8 = 19;

fn main() -> Result<(), PlatformError> {
    let game = DruidGoGame {
        game: Arc::new(RefCell::new(GoGameEngine::new(BOARD_SIZE))),
        version: 0,
    };

    let main_window = WindowDesc::new(ui_builder)
        .window_size((1000.0,1000.0));
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(game)
}

fn ui_builder() -> impl Widget<DruidGoGame> {
	return BoardWidget::new();
}
