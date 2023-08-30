extern crate sdl2;

use std::env;
use std::time::{Duration, SystemTime};

use crate::chip8::chip::Chip8;

mod audio;
mod chip8;
mod screen;

const FPS: u128 = 60;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect(); // check for argparser
    let rom_path = &args[1];
    let ipf: u32 = args[2].parse::<u32>().unwrap();

    let mut chip = Chip8::new();
    chip.read_rom(rom_path);

    while chip.is_running() {
        let start = SystemTime::now();

        chip.step(ipf);

        let sleep_for = start.elapsed().unwrap().as_nanos() + 1_000_000_000 / FPS;
        ::std::thread::sleep(Duration::new(0, sleep_for as u32));
    }

    Ok(())
}
