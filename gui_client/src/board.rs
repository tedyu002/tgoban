use druid::Data;
use druid::kurbo::{Line, Circle};
use druid::widget::prelude::*;
use druid::piet::{FontBuilder, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::{Affine, Color, LocalizedString, Point, Rect};
use druid::Event::{MouseDown, MouseUp};

use crate::game::DruidGoGame;
use go_board::{Location, ChessType};
        
const ChessRatio: f64 = 0.8;

pub struct BoardWidget {
    down_location: Location,
}

impl Widget<DruidGoGame> for BoardWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidGoGame, env: &Env) {
        let location = match event {
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
                            ctx.request_paint();
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
        if (!old_data.same(data)) {
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
        bc.max()
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
    fn paint_board(&mut self, ctx: &mut PaintCtx, data: &DruidGoGame, _env: &Env, game_size: u8, board_size: f64, chess_size: f64) {
        let chess_cen = chess_size / 2.0;

        for row in 1..=game_size {
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
        }

        for col in 1..=game_size {
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

                let circle = Circle::new(point, (chess_size/ 2.0 ) * ChessRatio);

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

        if distance > (chess_cen * chess_cen) * (ChessRatio * ChessRatio) {
            return Err(());
        }

        return Ok(Location {
            x: possible_location.0,
            y: game_size - 1 - possible_location.1,
        });
    }
}
