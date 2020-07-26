use druid::Data;
use druid::piet::{FontBuilder, Text, TextLayoutBuilder};
use druid::kurbo::{Line, Circle};
use druid::widget::prelude::*;
use druid::{Color, Point, Rect};
use druid::Event::{MouseDown, MouseUp};

use crate::game::DruidGoGame;
use go_board::{Location, ChessType};
        
const CHESS_RATIO: f64 = 0.8;

pub struct BoardWidget {
    down_location: Location,
}

impl Widget<DruidGoGame> for BoardWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidGoGame, env: &Env) {
        match event {
            MouseDown(mouse_event) => {
                let location = match self.get_location(ctx, data, env, mouse_event.pos) {
                    Ok(location) => location,
                    Err(_) => return,
                };
                self.down_location = location;
            },
            MouseUp(mouse_event) => {
                let location = match self.get_location(ctx, data, env, mouse_event.pos) {
                    Ok(location) => location,
                    Err(_) => return,
                };

                if self.down_location == location {
                    match data.game.borrow_mut().make_move(location) {
                        Ok(_) => {
                            data.version += 1;
                        },
                        Err(_) => {
                            return;
                        }
                    };
                }
            },
            _ => {
                return;
            }
        };
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &DruidGoGame, _env: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &DruidGoGame, data: &DruidGoGame, _env: &Env) {
        if !old_data.same(data) {
            ctx.request_paint();
        }
    }

	fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &DruidGoGame, _env: &Env) -> Size {
        // BoxConstraints are passed by the parent widget.
        // This method can return any Size within those constraints:
        // bc.constrain(my_size)
        //
        // To check if a dimension is infinite or not (e.g. scrolling):
        // bc.is_width_bounded() / bc.is_height_bounded()
        let size = bc.max();
        let size = size.width.min(size.height);

        Size {
            width: size,
            height: size,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidGoGame, env: &Env) {
		let size = ctx.size();

		let board_size = size.width.min(size.height);

		let rect = Rect::from_origin_size(Point::ORIGIN, druid::kurbo::Size {width: board_size, height: board_size,});
		ctx.fill(rect, &Color::rgb8(165, 116, 2));

        let game_size = data.game.borrow().size();
        let chess_size = board_size / (game_size + 2 /* Psudo for padding */) as f64;

        self.paint_board(ctx, data, env, game_size, board_size, chess_size);
        self.paint_chess(ctx, data, env, game_size, board_size, chess_size);
    }
}

impl BoardWidget {
    pub fn new() -> BoardWidget {
        BoardWidget {
            down_location: Location {
                x: 0,
                y: 0,
            }
        }
    }
}

impl BoardWidget {
    fn paint_board(&mut self, ctx: &mut PaintCtx, _data: &DruidGoGame, _env: &Env, game_size: u8, board_size: f64, chess_size: f64) {
        let chess_cen = chess_size / 2.0;

        let font = ctx
            .text()
            .new_font_by_name("Segoe UI", chess_cen)
            .build()
            .unwrap();

        for row in 1..=game_size {
            let font_layout = ctx
                .text()
                .new_text_layout(&font, &format!("{}", game_size - (row - 1)), std::f64::INFINITY)
                .build()
                .unwrap();

            let font_y = chess_size * row as f64 + chess_cen * 1.5;

            ctx.draw_text(&font_layout, (chess_cen, font_y), &Color::BLACK);

            let line = Line::new(
                Point{
                    x: chess_size + chess_cen,
                    y: chess_size * row as f64 + chess_cen,
                },
                Point{
                    x: board_size - chess_size - chess_cen,
                    y: chess_size * row as f64 + chess_cen,
                },
            );
            ctx.stroke(line, &Color::BLACK, 1.0);

            ctx.draw_text(&font_layout, (board_size - chess_size, font_y), &Color::BLACK);
        }

        for col in 1..=game_size {
            let alphabet = ('A' as u8 + (col - 1)) as char;

            let font_layout = ctx
                .text()
                .new_text_layout(&font, &format!("{}", alphabet), std::f64::INFINITY)
                .build()
                .unwrap();

            let font_x = chess_size * col as f64 + chess_cen * 0.75;

            ctx.draw_text(&font_layout, (font_x, chess_size), &Color::BLACK);

            let line = Line::new(
                Point{
                    x: chess_size * col as f64 + chess_cen,
                    y: chess_size + chess_cen,
                },
                Point{
                    x: chess_size * col as f64 + chess_cen,
                    y: board_size - chess_size - chess_cen,
                },
            );
            ctx.stroke(line, &Color::BLACK, 1.0);

            ctx.draw_text(&font_layout, (font_x, board_size - chess_cen), &Color::BLACK);
        }

        if game_size == 19 {
            let cross = [3, 9, 15];

            for &row in cross.iter() {
                for &col in cross.iter() {
                    let point = Point {
                        x: chess_size + col as f64 * chess_size + chess_cen,
                        y: board_size - chess_size - row as f64 * chess_size - chess_cen,
                    };

                    let circle = Circle::new(point, chess_size / 10.0);

                    ctx.fill(circle, &Color::BLACK);
                }
            }
        }
    }

    fn paint_chess(&mut self, ctx: &mut PaintCtx, data: &DruidGoGame, _env: &Env, game_size: u8, board_size: f64, chess_size: f64) {
        let chess_cen = chess_size / 2.0;

        for row in 0..game_size {
            for col in 0..game_size {
                let location = Location {
                    x: col,
                    y: row,
                };

                let color = match data.game.borrow().get_chess(location) {
                    ChessType::Black => {
                        Color::BLACK
                    },
                    ChessType::White => {
                        Color::WHITE
                    },
                    ChessType::None => {
                        continue;
                    },
                };

                let point = Point {
                    x: chess_size + col as f64 * chess_size + chess_cen,
                    y: board_size - chess_size - row as f64 * chess_size - chess_cen,
                };

                let circle = Circle::new(point, (chess_size / 2.0) * CHESS_RATIO);

                ctx.fill(circle, &color);
            }
        }
    }

    fn get_location(&mut self, ctx: &mut EventCtx, data: &mut DruidGoGame, _env: &Env, point: Point) -> Result<Location, ()> {
		let size = ctx.size();
		let board_size = size.width.min(size.height);
        let game_size = data.game.borrow().size();
        let chess_size = board_size / (game_size + 2 /* Psudo for padding */) as f64;
        let chess_cen = chess_size / 2.0;

        let original = Point {
            x: chess_size + chess_cen,
            y: chess_size + chess_cen,
        };

        let point = Point {
            x: point.x - original.x,
            y: point.y - original.y,
        };

        let possible_location = ((point.x / chess_size).round() as i8, (point.y / chess_size).round() as i8);

        if possible_location.0 < 0 || possible_location.1 < 0 {
            return Err(());
        }

        let possible_location = (possible_location.0 as u8, possible_location.1 as u8);
        if possible_location.0 >= game_size || possible_location.1 >= game_size {
            return Err(());
        }

        let chess_center = (possible_location.0 as f64 * chess_size,
                            possible_location.1 as f64 * chess_size);

        let distance = (point.x - chess_center.0, point.y - chess_center.1);
        let distance = distance.0 * distance.0 + distance.1 * distance.1;

        if distance > (chess_cen * chess_cen) * (CHESS_RATIO * CHESS_RATIO) {
            return Err(());
        }

        return Ok(Location {
            x: possible_location.0,
            y: game_size - 1 - possible_location.1,
        });
    }
}
