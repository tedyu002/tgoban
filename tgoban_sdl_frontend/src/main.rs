mod components;

use std::sync::{Arc, RwLock};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

use sdl2::gfx::primitives::DrawRenderer;

use go_game_engine::GoGameEngine;

use components::Drawable;
use components::Board;


fn main() -> Result<(), String> {
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "1");

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let mut lastx = 0;
    let mut lasty = 0;

    let mut events = sdl_context.event_pump()?;

    let mut coms: Vec<(Box<dyn components::Drawable>, WindowCanvas)> = Vec::new();

    {
        let board = Board::new(Arc::new(RwLock::new(GoGameEngine::new(19, 6.5))));
        let canvas = board.build_canvas(&video_subsys);
        coms.push((
            Box::new(board),
            canvas
        ));
    }

    'main: loop {
        for event in events.wait_iter() {
            match event {
                Event::Quit {..} => break 'main,

                Event::MouseButtonDown {x, y, window_id, ..} => {
                    for com in coms.iter_mut() {
                        if com.1.window().id() == window_id {
                            com.0.handle_event(&mut com.1, event);
                            continue 'main;
                        }
                    }
                }
/*

                Event::KeyDown {keycode: Some(keycode), ..} => {
                    if keycode == Keycode::Escape {
                        break 'main
                    } else if keycode == Keycode::Space {
                        println!("space down");
                        for i in 0..400 {
                            canvas.pixel(i as i16, i as i16, 0xFF000FFu32)?;
                        }
                        canvas.present();
                    }
                }
*/

                _ => {}
            }
        }
    }

    Ok(())
}
