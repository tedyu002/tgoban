use std::sync::Arc;
use std::cell::RefCell;

use druid::Env;
use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};

use go_game_engine::GoGameEngine;
use go_board::{Location, GoBoard, ChessType};


const BOARD_SIZE: u8 = 19;

#[derive(Clone)]
struct DruidGoGame {
    game: Arc<RefCell<GoGameEngine>>,
    version: i32,
}

impl druid::Data for DruidGoGame {
    fn same(&self, other: &Self) -> bool {
        return self.version == other.version;
    }
}

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
    let mut board = Flex::column();

    let make_number_row = |size| {
        let mut row = Flex::row();

        row = row.with_child(Button::new(""));
        for i in 0..size {
            row = row.with_child(Button::new(format!("{}", ('A' as u8 + i) as char)));
        }

        row.with_child(Button::new(""))
    };


    board = board.with_child(make_number_row(BOARD_SIZE));

    for i in 0..BOARD_SIZE {
        let mut row = Flex::row();

        row = row.with_child(Label::new(format!("{}", BOARD_SIZE - i)));
        for j in 0..BOARD_SIZE {
            let location = Location {
                x: i,
                y: j,
            };
            let button = Button::new(move |druid_game: &DruidGoGame, _: &Env| {
                    match druid_game.game.borrow().get_chess(location) {
                        ChessType::Black => String::from("O"),
                        ChessType::White => String::from("X"),
                        ChessType::None => String::from("."),
                    }
                })
                .on_click(move |_ctx, druid_game, _env| {
                    druid_game.game.borrow_mut().make_move(location);
                    druid_game.version += 1;
                });
            row = row.with_child(button);

        }
        row = row.with_child(Label::new(format!("{}", BOARD_SIZE - i)));

        board = board.with_child(row);
    }
    board = board.with_child(make_number_row(BOARD_SIZE));

    return board;
}
