mod board;
mod game;
mod game_info;

use std::sync::Arc;
use std::cell::RefCell;

use druid::widget::Flex;
use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use go_game_engine::GoGameEngine;

use game::DruidGoGame;
use board::BoardWidget;
use game_info::make_game_info_widget;

const BOARD_SIZE: u8 = 19;

fn main() -> Result<(), PlatformError> {
    let game = DruidGoGame {
        game: Arc::new(RefCell::new(GoGameEngine::new(BOARD_SIZE))),
        version: 0,
    };

    let main_window = WindowDesc::new(ui_builder)
        .window_size((1500.0,1000.0));
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(game)
}

fn ui_builder() -> impl Widget<DruidGoGame> {
    Flex::row()
        .with_child(BoardWidget::new())
        .with_child(make_game_info_widget())
}
