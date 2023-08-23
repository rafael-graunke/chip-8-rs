use crate::chip8::Chip8;
use std::env;

mod chip8;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];

    let mut chip = Chip8::new();

    chip.read_rom(rom_path);

    loop {
        chip.step();
    }
}
