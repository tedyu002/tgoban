use druid::widget::{Label, Flex};

use go_game_engine::Player;
use crate::game::DruidGoGame;


pub fn make_game_info_widget() -> Flex<DruidGoGame> {
    let player = Label::new("Player");
    let player_info = Label::new(|data: &DruidGoGame, _env: &_| {
        format!("{}",
            match data.game.borrow().player() {
                Player::Black => "Black",
                Player::White => "White",
            }
        )
    });

    let black = Label::new("Black");
    let black_deads = Label::new(|data: &DruidGoGame, _env: &_| {
        format!("{}", data.game.borrow().get_capture(&Player::Black))
    });

    let white = Label::new("White");
    let white_deads = Label::new(|data: &DruidGoGame, _env: &_| {
        format!("{}", data.game.borrow().get_capture(&Player::White))
    });

    Flex::column()
        .with_child(
            Flex::row()
                .with_child(player)
                .with_child(player_info)
        )
        .with_child(
            Flex::row()
                .with_child(black)
                .with_child(black_deads)
        )
        .with_child(
            Flex::row()
                .with_child(white)
                .with_child(white_deads)
        )
}
