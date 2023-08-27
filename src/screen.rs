use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const PIXEL_SCALE: usize = 20;

pub struct Display {
    pub screen_memory: [u64; SCREEN_HEIGHT],
    pub canvas: Canvas<Window>,
}

impl Display {
    fn init_canvas(sdl: &Sdl) -> Canvas<Window> {
        let video_subsystem = sdl.video().unwrap();

        let window = video_subsystem
            .window(
                "rust-sdl2 demo: Video",
                SCREEN_WIDTH as u32 * PIXEL_SCALE as u32,
                SCREEN_HEIGHT as u32 * PIXEL_SCALE as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(Color::RGB(38, 17, 13));
        canvas.clear();

        canvas
    }

    pub fn new(sdl: &Sdl) -> Display {
        Display {
            screen_memory: [0u64; SCREEN_HEIGHT],
            canvas: Display::init_canvas(sdl),
        }
    }

    pub fn clear(&mut self) {
        self.screen_memory = [0u64; SCREEN_HEIGHT];
    }

    pub fn render(&mut self) {
        let mut pixel = Rect::new(0, 0, PIXEL_SCALE as u32, PIXEL_SCALE as u32);

        let width = SCREEN_WIDTH as u64;

        for (row_index, row) in self.screen_memory.iter().enumerate() {
            for column in 0..width {
                if 1u64 << (width - 1 - column) & row != 0 {
                    pixel.x = column as i32 * PIXEL_SCALE as i32;
                    pixel.y = row_index as i32 * PIXEL_SCALE as i32;
                    self.canvas.set_draw_color(Color::RGB(155, 66, 49));
                    self.canvas.fill_rect(pixel).unwrap();
                }
            }
        }

        self.canvas.present();
    }
}
