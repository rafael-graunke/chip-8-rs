extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, SystemTime};

use crate::chip8::Chip8;
use std::env;

mod chip8;
mod screen;

const FPS: u128 = 60;

fn main() -> Result<(), String> {
    // SDL2 Setup
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;

    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];
    let ipf: u32 = args[2].parse::<u32>().unwrap();

    let mut chip = Chip8::new(&sdl_context);
    chip.read_rom(rom_path);

    'running: loop {
        let start = SystemTime::now();

        for _ in 0..ipf {
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
        }

        chip.render();

        let sleep_for = start.elapsed().unwrap().as_nanos() + 1_000_000_000 / FPS;
        ::std::thread::sleep(Duration::new(0, sleep_for as u32));
    }

    Ok(())
}
