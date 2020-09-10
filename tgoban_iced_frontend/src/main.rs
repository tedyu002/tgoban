mod widgets;

use std::rc::Rc;
use std::cell::RefCell;

use iced_native::{Widget};
use iced_wgpu::Renderer;
use iced::{
    Element, Settings,
    Color, Point, Application, executor,
    Command, Length, Size, Subscription,
    Text,
    canvas::{
        self, Canvas,
        Path,
    },
    widget::{
        Row, Column,
        canvas::Stroke,
    }
};

use go_game_engine::{GoGameEngine, ChessType, Location};

fn main() {
    App::run(Settings{
        antialiasing: true,
        ..Settings::default()
    });
}

struct App {
    go_game: Rc<RefCell<GoGameEngine>>,
    state: crate::widgets::board::State,
    pass_button_state: iced_native::widget::button::State,
}

#[derive(Debug, Clone)]
enum Message {
    Play(Location),
    Back,
    Pass,
    None,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let go_game = Rc::new(RefCell::new(GoGameEngine::new(19, 6.5)));

        (
            App {
                go_game: go_game.clone(),
                state: crate::widgets::board::State::new(go_game.clone()),
                pass_button_state: iced_native::widget::button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Example")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Play(location) => {
                let mut go_game = self.go_game.borrow_mut();

                match go_game.get_status() {
                    go_game_engine::GameStatus::Playing => {
                        go_game.make_move(location);
                    },
                    go_game_engine::GameStatus::Scoring => {
                        go_game.toggle(location);
                    },
                };
            },
            Message::Back => {
                self.go_game.borrow_mut().regret();
            },
            Message::Pass => {
                self.go_game.borrow_mut().pass();
            },
            Message::None => {
            },
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let go_game = self.go_game.borrow();

        let mut row = Row::new();

        row = row.push(
            crate::widgets::board::Board::new(
                &mut self.state
            )
            .on_play(
                | location | {
                    Message::Play(Location {
                        alphabet: location.letter,
                        digit: location.number,
                    })
                }
            )
            .on_back(
                | | {
                    Message::Back
                }
            )
        );

        let mut panel =
            Column::new()
            .push(
                Text::new(format!("Playing: {}", match go_game.player() {
                    go_game_engine::Player::Black => "Black",
                    go_game_engine::Player::White => "White",
                }))
            )
            .push(
                Text::new(format!("Komi: {}", go_game.komi()))
            )
            .push(
                Text::new(format!("Steps: {}", go_game.steps()))
            )
            .push(
                Text::new(format!("Black Captuer: {}", go_game.get_capture(&go_game_engine::Player::Black)))
            )
            .push(
                Text::new(format!("White Captuer: {}", go_game.get_capture(&go_game_engine::Player::White)))
            )
            .push(
                iced_native::widget::button::Button::new(
                    &mut self.pass_button_state,
                    Text::new("Pass")
                )
                .on_press(Message::Pass)
            );

        match go_game.get_status() {
            go_game_engine::GameStatus::Scoring => {
                let score = go_game.get_score();

                panel = panel
                .push(
                    Text::new(format!("Black Score: {}", score.0))
                )
                .push(
                    Text::new(format!("White Score: {}", score.1))
                );
            },
            go_game_engine::GameStatus::Playing => {
            },
        };

        row = row.push(panel);

        row.into()
    }
}
