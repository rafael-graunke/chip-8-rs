use std::env;
use std::fs::File;
use std::io::prelude::*;

fn read_rom(path: &String) -> Vec<u8> {
    let mut file = File::open(&path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    data
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];

    let rom = read_rom(rom_path);

    println!("{rom:?}");
}
