use std::sync::{Arc, RwLock};

use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::gfx::primitives::ToColor;
use sdl2::pixels::Color;
use sdl2::event::Event;

use sdl2::gfx::primitives::DrawRenderer;

use go_game_engine::{GoGameEngine, ChessType, Location};

use super::Drawable;

pub struct Board {
    go_game_engine_holder: Arc<RwLock<GoGameEngine>>,
}

impl Board {
    pub fn new(go_game_engine_holder: Arc<RwLock<GoGameEngine>>) -> Board {
        Board {
            go_game_engine_holder,
        }
    }
}
/*
const BOARD_SIZE: u8 = 19;
const CHESS_SIZE: i32 = 50;
const AREA_NUM: i32 = BOARD_SIZE as i32 + 2;
const CANVAS_SIZE: i32 = CHESS_SIZE * AREA_NUM;
const LINE_START: i32 = CHESS_SIZE + CHESS_SIZE / 2;
const LINE_END: i32 = CANVAS_SIZE - LINE_START;
const LINE_WIDTH: u8 = 1;
const FONT_SIZE: i32 = CHESS_SIZE / 2;
*/
impl Board { /* For the draw */
    fn draw_empty(&self, canvas: &mut WindowCanvas) {
        let output_size = canvas.output_size().unwrap();
        let canvas_size = std::cmp::min(output_size.0, output_size.1);

        let BOARD_SIZE: u8 = 19;
        let CHESS_SIZE: i32 = (canvas_size / 21) as i32;
        let AREA_NUM: i32 = BOARD_SIZE as i32 + 2;
        let CANVAS_SIZE: i32 = CHESS_SIZE * AREA_NUM;
        let LINE_START: i32 = CHESS_SIZE + CHESS_SIZE / 2;
        let LINE_END: i32 = CANVAS_SIZE - LINE_START;
        let LINE_WIDTH: u8 = 1;
        let FONT_SIZE: i32 = CHESS_SIZE / 2;        
        canvas.set_draw_color((165u8, 116u8, 2u8, 255u8));
        canvas.fill_rect(Rect::new(0, 0, CANVAS_SIZE as u32, CANVAS_SIZE as u32));

        let black: Color = (0u8, 0u8, 0u8, 255u8).into();
        canvas.set_draw_color(black);

        for row in 0..(BOARD_SIZE as i32) {
            canvas.thick_line((LINE_START + CHESS_SIZE * row) as i16, LINE_START as i16,
                            (LINE_START + CHESS_SIZE * row) as i16, LINE_END as i16,
                            LINE_WIDTH, black);
        }

        for col in 0..(BOARD_SIZE as i32) {
            canvas.thick_line(LINE_START as i16, (LINE_START + CHESS_SIZE * col) as i16,
                           LINE_END as i16, (LINE_START + CHESS_SIZE * col) as i16,
                           LINE_WIDTH, black);
        }
    }

    fn draw_chess(&self, canvas: &mut WindowCanvas, go_game: &GoGameEngine) {
        let output_size = canvas.output_size().unwrap();
        let canvas_size = std::cmp::min(output_size.0, output_size.1);

        let BOARD_SIZE: u8 = 19;
        let CHESS_SIZE: i32 = (canvas_size / 21) as i32;
        let AREA_NUM: i32 = BOARD_SIZE as i32 + 2;
        let CANVAS_SIZE: i32 = CHESS_SIZE * AREA_NUM;
        let LINE_START: i32 = CHESS_SIZE + CHESS_SIZE / 2;
        let LINE_END: i32 = CANVAS_SIZE - LINE_START;
        let LINE_WIDTH: u8 = 1;
        let FONT_SIZE: i32 = CHESS_SIZE / 2;        

        for alphabet in 0..go_game.size() {
            for digit in 0..go_game.size() {
                let location = Location {
                    alphabet,
                    digit,
                };

                let chess = go_game.get_chess(location);

                let color = match chess {
                    ChessType::Black => (0u8, 0u8, 0u8, 255u8),
                    ChessType::White => (255u8, 255u8, 255u8, 255u8),
                    ChessType::None => {
                        continue;
                    },
                };
                let chess_center = to_chess_center(go_game.size(), CHESS_SIZE, alphabet, digit);
                canvas.filled_circle(chess_center.0 as i16, chess_center.1 as i16, (CHESS_SIZE / 2) as i16, color);
            }
        }
    }
}

fn to_chess_center(board_size: u8, CHESS_SIZE: i32, alphabet: u8, digit: u8) -> (i32, i32) {
    (
        (CHESS_SIZE + CHESS_SIZE / 2) as i32 + (alphabet as i32) * CHESS_SIZE,
        (CHESS_SIZE + CHESS_SIZE / 2) as i32 + ((board_size - digit - 1) as i32) * CHESS_SIZE
    )
}

fn convert_location(container: (i32, i32), offset: (i32, i32)) -> Option<(u8, u8)> {
    let BOARD_SIZE: u8 = 19;
    let CHESS_SIZE: i32 = (container.0 / 21) as i32;
    let AREA_NUM: i32 = BOARD_SIZE as i32 + 2;
    let CANVAS_SIZE: i32 = CHESS_SIZE * AREA_NUM;
    let LINE_START: i32 = CHESS_SIZE + CHESS_SIZE / 2;
    let LINE_END: i32 = CANVAS_SIZE - LINE_START;
    let LINE_WIDTH: u8 = 1;
    let FONT_SIZE: i32 = CHESS_SIZE / 2;

    let x = (offset.0 / (container.0 / AREA_NUM)) as u8;
    let y = (offset.1 / (container.1 / AREA_NUM)) as u8;

    if x == 0 || y == 0 || x > BOARD_SIZE || y > BOARD_SIZE {
        return None;
    } else {
        return Some((x as u8 - 1, BOARD_SIZE - (y as u8)));
    }
}

impl Drawable for Board {
    fn draw(&self, canvas: &mut WindowCanvas) {

        canvas.set_draw_color((0u8, 0u8, 0u8, 255u8));
        canvas.clear();
        self.draw_empty(canvas);
        {
            let lock_result = self.go_game_engine_holder.read().expect("Get read lock error");
            self.draw_chess(canvas, &*lock_result);
        }

        canvas.present();
    }

    fn handle_event(&mut self, canvas: &mut WindowCanvas, event: Event) {

        match event {
            Event::MouseButtonDown {x, y, ..} => {
                let output_size = canvas.output_size().unwrap();
                let canvas_size = std::cmp::min(output_size.0, output_size.1);
                let location = convert_location((canvas_size as i32, canvas_size as i32), (x, y)).unwrap();
                {
                    let mut lock_result = self.go_game_engine_holder.write().expect("Get read lock error");
                    lock_result.make_move(Location{
                        alphabet: location.0,
                        digit: location.1,
                    });
                }
            },
            _ => {
                return;
            }
        }

        self.draw(canvas);
    }
}
