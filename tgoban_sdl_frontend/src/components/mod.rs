mod board;

pub use board::Board;

use sdl2::VideoSubsystem;
use sdl2::render::WindowCanvas;
use sdl2::pixels;
use sdl2::event::Event;

const SCREEN_WIDTH: u32 = 1000;
const SCREEN_HEIGHT: u32 = 1000;

pub trait Drawable {
    fn draw(&self, canvas: &mut WindowCanvas);
    fn handle_event(&mut self, canvas: &mut WindowCanvas, event: Event);

    fn build_canvas(&self, video_subsys: &VideoSubsystem) -> WindowCanvas {
        let window = video_subsys.window("rust-sdl2_gfx: draw line & FPSManager", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .resizable()
            .allow_highdpi()
            .vulkan()
            .build()
            .expect("Failed to build windows");

        let mut canvas = window.into_canvas().build().expect("Failed to build canvas");

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        canvas
    }
}
