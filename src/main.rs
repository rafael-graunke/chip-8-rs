use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
struct Chip8 {
    memory: Vec<u8>,
    stack: Vec<u8>,
    display: [u64; 32], // change later
    registers: [u8; 16],
    vi: u16,
}

impl Chip8 {
    fn font_data() -> Vec<u8> {
        vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]
    }
    fn new() -> Chip8 {
        Chip8 {
            memory: Chip8::font_data(), // initialize with fonts in memory
            stack: vec![],
            display: [0u64; 32],
            registers: [0u8; 16],
            vi: 0u16,
        }
    }

    fn read_rom(&mut self, path: &String) {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        self.memory.append(&mut data); // loads rom to ram
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];

    let mut chip = Chip8::new();

    chip.read_rom(rom_path);

    println!("{:?}", chip);
}
