use std::rc::Rc;
use std::cell::RefCell;

use iced_native::{
    input, layout, Clipboard, Color, Element, Event, Font, Hasher,
    HorizontalAlignment, Layout, Length, MouseCursor, Point, Rectangle,
    Size, Vector, VerticalAlignment, Widget, Background,
};
use iced_wgpu::{
    triangle::{Mesh2D, Vertex2D},
    Defaults, Primitive, Renderer,
};

use go_game_engine::GoGameEngine;

pub struct State {
    go_game: Rc<RefCell<GoGameEngine>>,
    mouse_location: Point,
}

impl State {
    pub fn new(go_game: Rc<RefCell<GoGameEngine>>) -> Self {
        State {
            go_game,
            mouse_location: [0, 0].into(),
        }
    }
}

pub struct Location {
    pub letter: u8,
    pub number: u8,
}

pub struct Board<'a, Message> {
    state: &'a mut State,
    on_play: Option<Box<dyn Fn(Location) -> Message>>,
    on_back: Option<Box<dyn Fn() -> Message>>,
}

impl<'a, Message> Board<'a, Message> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            on_play: None,
            on_back: None,
        }
    }

    pub fn on_play<F> (
        mut self,
        action: F,
    ) -> Self
    where
        F: 'static + Fn(Location) -> Message,
    {
        self.on_play = Some(Box::new(action));
        self
    }

    pub fn on_back<F> (
        mut self,
        action: F,
    ) -> Self
    where
        F: 'static + Fn() -> Message,
    {
        self.on_back = Some(Box::new(action));
        self
    }
}

impl<'a, Message> Widget<Message, Renderer> for Board<'a, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits
            .height(Length::Fill)
            .width(Length::Fill)
            .resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn hash_layout(&self, _state: &mut Hasher) {
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        let bounds = layout.bounds();

        println!("{:#?}", cursor_position);

        if bounds.contains(cursor_position) {
            println!("{:#?}", event);
            match event {
                iced_native::Event::Mouse(mouse_event) => match mouse_event {
                    iced_native::input::mouse::Event::CursorMoved {x, y} => {
                        self.state.mouse_location = [x, y].into();
                    },
                    iced_native::input::mouse::Event::Input {state, button} => {
                        match state {
                            iced_native::input::ButtonState::Released => {
                                match button {
                                    iced_native::input::mouse::Button::Left => {
                                        println!("{:#?}", event);

                                        let board_canvas_size = BoardCanvasSize::from(bounds);

                                        match board_canvas_size.convert_location(cursor_position) {
                                            None => {},
                                            Some(location) => {
                                                if let Some(action) = &self.on_play {
                                                    messages.push(action(location));
                                                }
                                            },
                                        };
                                    },
                                    iced_native::input::mouse::Button::Right => {
                                        if let Some(action) = &self.on_back {
                                            messages.push(action());
                                        }
                                    },
                                    _ => {
                                    },
                                };
                            },
                            iced_native::input::ButtonState::Pressed => {
                            },
                        };
                    },
                    _ => {},
                },
                _ => {},
            };
        }
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        defaults: &Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> (Primitive, MouseCursor) {
        let board_canvas_size = BoardCanvasSize::from(layout.bounds());

        println!("Draw {:#?}", layout.bounds());

        let mut primitives: Vec<Primitive> = Vec::new();

        self.draw_empty(&board_canvas_size, &mut primitives);
        self.draw_chess(&board_canvas_size, &mut primitives);
        self.draw_belong(&board_canvas_size, &mut primitives);

        (
            Primitive::Group {
                primitives: primitives,
            },
            MouseCursor::OutOfBounds,
        )
    }
}

impl<'a, Message> Board<'a, Message> {
    fn draw_empty(
        &self,
        board_canvas_size: &BoardCanvasSize,
        primitives: &mut Vec<Primitive>
    ) {
        primitives.push(
            Primitive::Quad {
                bounds: Rectangle {
                    x: board_canvas_size.canvas_offset.x,
                    y: board_canvas_size.canvas_offset.y,
                    width: board_canvas_size.canvas_size,
                    height: board_canvas_size.canvas_size,
                },
                background: Background::Color(Color::from_rgb8(165u8, 116u8, 2u8)),
                border_radius: 0u16,
                border_width: 0u16,
                border_color: Color::BLACK,
            },
        );
        for letter in 0..(board_canvas_size.board_size) {
            let start_point = board_canvas_size.get_point(letter, board_canvas_size.board_size - 1);
            let end_point = board_canvas_size.get_point(letter, 0u8);

            primitives.push(
                Primitive::Quad {
                    bounds: Rectangle {
                        x: start_point.x, /* TODO */
                        y: start_point.y,
                        width: board_canvas_size.line_width,
                        height: end_point.y - start_point.y, /* TODO */
                    },
                    background: Background::Color(Color::BLACK),
                    border_radius: 0u16,
                    border_width: 0u16,
                    border_color: Color::BLACK,
                }
            );
        }

        for number in 0..(board_canvas_size.board_size) {
            let start_point = board_canvas_size.get_point(0u8, number);
            let end_point = board_canvas_size.get_point(board_canvas_size.board_size - 1, number);

            primitives.push(
                Primitive::Quad {
                    bounds: Rectangle {
                        x: start_point.x, /* TODO */
                        y: start_point.y,
                        width: end_point.x - start_point.x, /* TODO */
                        height: board_canvas_size.line_width,
                    },
                    background: Background::Color(Color::BLACK),
                    border_radius: 0u16,
                    border_width: 0u16,
                    border_color: Color::BLACK,
                }
            );
        }
    }

    fn draw_chess(
        &self,
        board_canvas_size: &BoardCanvasSize,
        primitives: &mut Vec<Primitive>
    ) {
        let go_game = self.state.go_game.borrow();

        for alphabet in 0..go_game.size() {
            for digit in 0..go_game.size() {
                let location = go_game_engine::Location {
                    alphabet,
                    digit,
                };

                let chess = go_game.get_chess(location);

                let color = match chess {
                    go_game_engine::ChessType::Black => Color::BLACK,
                    go_game_engine::ChessType::White => Color::WHITE,
                    go_game_engine::ChessType::None => {
                        continue;
                    },
                };

                let color = match go_game.is_alive(location) {
                    true => Color {
                        a: 1f32,
                        ..color
                    },
                    false => Color {
                        a: 0.5f32,
                        ..color
                    },
                };
                let center = board_canvas_size.get_point(alphabet, digit);

                primitives.push(
                    Primitive::Quad {
                        bounds: Rectangle {
                            x: center.x - board_canvas_size.chess_draw_size / 2f32, /* TODO */
                            y: center.y - board_canvas_size.chess_draw_size / 2f32,
                            width: board_canvas_size.chess_draw_size,
                            height: board_canvas_size.chess_draw_size,
                        },
                        background: Background::Color(color),
                        border_radius: (board_canvas_size.chess_draw_size / 2f32) as u16,
                        border_width: 0u16,
                        border_color: Color::BLACK,
                    },
                );
            }
        }
    }

    fn draw_belong(
        &self,
        board_canvas_size: &BoardCanvasSize,
        primitives: &mut Vec<Primitive>
    ) {
        let go_game = self.state.go_game.borrow();

        for alphabet in 0..go_game.size() {
            for digit in 0..go_game.size() {
                let location = go_game_engine::Location {
                    alphabet,
                    digit,
                };

                let belong = match go_game.get_belong(location) {
                    None => continue,
                    Some(player) => match player {
                        go_game_engine::Player::Black => Color::BLACK,
                        go_game_engine::Player::White => Color::WHITE,
                    },
                };

                let center = board_canvas_size.get_point(alphabet, digit);

                primitives.push(
                    Primitive::Quad {
                        bounds: Rectangle {
                            x: center.x - board_canvas_size.chess_draw_size / 4f32, /* TODO */
                            y: center.y - board_canvas_size.chess_draw_size / 4f32,
                            width: board_canvas_size.chess_draw_size / 2f32,
                            height: board_canvas_size.chess_draw_size / 2f32,
                        },
                        background: Background::Color(belong),
                        border_radius: 0u16,
                        border_width: 0u16,
                        border_color: belong,
                    },
                );
            }
        }
    }
}

impl<'a, Message> Into<Element<'a, Message, Renderer>> for Board<'a, Message>
where
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}

struct BoardCanvasSize {
    canvas_offset: Point,
    canvas_size: f32,
    board_size: u8,
    area_num: i32,
    chess_size: f32,
    chess_draw_size: f32,
    chess_click_size: f32,
    line_start: f32,
    line_end: f32,
    line_width: f32,
    font_size: f32,
}

impl BoardCanvasSize {
    fn new(offset: Point, size: Size) -> BoardCanvasSize {
        let canvas_size = match size.width < size.height {
            true => size.width,
            false => size.height,
        };

        let canvas_size = (canvas_size / 21f32).ceil() * 21f32;

        let board_size: u8 = 19;
        let area_num: i32 = board_size as i32 + 2;
        let chess_size: f32 = canvas_size / (area_num as f32);
        let line_start: f32 = chess_size + chess_size / 2f32;
        let line_end: f32 = canvas_size - line_start;
        let line_width: f32 = 1f32;
        let font_size: f32 = chess_size / 2f32;

        BoardCanvasSize {
            canvas_offset: offset,
            canvas_size,
            board_size,
            area_num,
            chess_size,
            chess_draw_size: chess_size * 0.9f32,
            chess_click_size: chess_size * (0.5f32 * 0.8f32),
            line_start,
            line_end,
            line_width,
            font_size,
        }
    }

    fn get_point(&self, letter: u8, number: u8) -> Point {
        let get_single_point = |num: u8| {
            self.line_start + self.chess_size * (num as f32)
        };

        [
            get_single_point(letter) + self.canvas_offset.x,
            get_single_point(self.board_size - 1 - number) + self.canvas_offset.y
        ].into()
    }

    fn convert_location(&self, point: Point) -> Option<Location> {
        let x = ((point.x - self.canvas_offset.x) / (self.canvas_size / self.area_num as f32)) as u8;
        let y = ((point.y - self.canvas_offset.y) / (self.canvas_size / self.area_num as f32)) as u8;

        if x == 0 || y == 0 || x > self.board_size || y > self.board_size {
            return None;
        } else {
            let location = Location {
                letter: x - 1,
                number: self.board_size - y
            };

            let center_point = self.get_point(location.letter, location.number);

            let distance = (point.x - center_point.x) * (point.x - center_point.x) +
                            (point.y - center_point.y) * (point.y - center_point.y);
            let distance_limit = self.chess_click_size * self.chess_click_size;

            match distance < distance_limit {
                true => Some(location),
                false => None,
            }
        }
    }
}

impl From<Rectangle<f32>> for BoardCanvasSize {
    fn from(rectangle: Rectangle<f32>) -> Self {
        let small_size = match rectangle.width < rectangle.height {
            true => rectangle.width,
            false => rectangle.height,
        };

        BoardCanvasSize::new(
            [rectangle.x, rectangle.y].into(),
            [small_size, small_size].into()
        )
    }
}
