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
    pc: u16,
    did_jump: bool,
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
            pc: Chip8::font_data().len() as u16, // starts after fonts
            did_jump: false,
        }
    }

    fn read_rom(&mut self, path: &String) {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        self.memory.append(&mut data); // loads rom to ram
    }

    // might change this later
    fn get_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16)
    }

    fn run_opcode(&self, opcode: u16) {
        // for single nibble determinant
        let mut mask = 0xF000;

        match opcode & mask {
            0x1000 => println!("JP addr"),
            0x2000 => println!("CALL addr"),
            0x3000 => println!("SE Vx, byte"),
            0x4000 => println!("SNE Vx, byte"),
            0x5000 => println!("SE Vx, Vy"),
            0x6000 => println!("LD Vx, byte"),
            0x7000 => println!("ADD Vx, byte"),
            0x9000 => println!("SNE Vx, Vy"),
            0xA000 => println!("LD I, addr"),
            0xB000 => println!("JP V0, addr"),
            0xC000 => println!("RND Vx, byte"),
            0xD000 => println!("DRW Vx, Vy, nibble"),
            _ => {}
        }

        // for dual nibble determinant
        mask = 0xF00F;

        match opcode & mask {
            0x0000 => println!("CLS"),
            0x000E => println!("RET"),
            0x8000 => println!("LD Vx, Vy"),
            0x8001 => println!("OR Vx, Vy"),
            0x8002 => println!("AND Vx, Vy"),
            0x8003 => println!("XOR Vx, Vy"),
            0x8004 => println!("ADD Vx, Vy"),
            0x8005 => println!("SUB Vx, Vy"),
            0x8006 => println!("SHR Vx {{, Vy}}"),
            0x8007 => println!("SUBN Vx, Vy"),
            0x800E => println!("SHL Vx {{, Vy}}"),
            0xE00E => println!("SKP Vx"),
            0xE001 => println!("SKNP Vx1"),
            _ => {}
        }

        //for F codes
        mask = 0xF0FF;

        match opcode & mask {
            0xF007 => println!("LD Vx, DT"),
            0xF00A => println!("LD Vx, K"),
            0xF015 => println!("LD DT, Vx"),
            0xF018 => println!("LD ST, Vx"),
            0xF01E => println!("ADD I, Vx"),
            0xF029 => println!("LD F, Vx"),
            0xF033 => println!("LD B, Vx"),
            0xF055 => println!("LD [I], Vx"),
            0xF065 => println!("LD Vx, [I]"),
            _ => {}
        }
    }

    fn step(&mut self) {
        let opcode = self.get_opcode();

        self.run_opcode(opcode);

        if !self.did_jump {
            self.pc += 2;
            self.did_jump = false;
        };
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];

    let mut chip = Chip8::new();

    chip.read_rom(rom_path);

    loop {
        chip.step();
    }
}
