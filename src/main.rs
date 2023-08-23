extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::{event::Event, render::Canvas};
use std::time::Duration;

use crate::chip8::{Chip8, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::env;

mod chip8;

const PIXEL_SCALE: usize = 20;

fn render(canvas: &mut Canvas<Window>, chip: &Chip8) {
    let mut pixel = Rect::new(0, 0, PIXEL_SCALE as u32, PIXEL_SCALE as u32);

    let width = SCREEN_WIDTH as u64;

    for (j, row) in chip.display.iter().enumerate() {
        for i in 0..width {
            if 1u64 << (width - 1 - i) & row != 0 {
                pixel.x = i as i32 * PIXEL_SCALE as i32;
                pixel.y = j as i32 * PIXEL_SCALE as i32;
                canvas.set_draw_color(Color::RGB(155, 66, 49));
                canvas.fill_rect(pixel).unwrap();
            }
        }
    }

    canvas.present();
}

fn main() -> Result<(), String> {
    /* SDL2 Setup */
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "rust-sdl2 demo: Video",
            (SCREEN_WIDTH as u32 * PIXEL_SCALE as u32) as u32,
            (SCREEN_HEIGHT as u32 * PIXEL_SCALE as u32) as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(38, 17, 13));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    /* Chip8 Setup */

    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];

    let mut chip = Chip8::new();
    chip.read_rom(rom_path);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        chip.step();

        if chip.should_draw {
            render(&mut canvas, &chip);
            chip.should_draw = false;
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
