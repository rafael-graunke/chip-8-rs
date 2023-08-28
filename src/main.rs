extern crate sdl2;

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

    let mut chip = Chip8::new(&sdl_context, &mut event_pump);
    chip.read_rom(rom_path);

    let mut running = true;

    while running {
        let start = SystemTime::now();

        running = chip.step(ipf);
        chip.render();

        let sleep_for = start.elapsed().unwrap().as_nanos() + 1_000_000_000 / FPS;
        ::std::thread::sleep(Duration::new(0, sleep_for as u32));
    }

    Ok(())
}
