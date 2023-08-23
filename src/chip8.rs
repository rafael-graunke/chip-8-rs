use std::fs::File;
use std::io::prelude::*;

pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: usize = 32;
const MEM_OFFSET: u16 = 0x200;
const FONT_OFFSET: u16 = 0x50;

pub struct Chip8 {
    memory: Vec<u8>,
    stack: Vec<u8>,
    pub display: [u64; SCREEN_HEIGHT], // change later
    registers: [u8; 16],
    fonts: [u8; 80],
    vi: u16,
    pc: u16,
    did_jump: bool,
    pub should_draw: bool,
}

impl Chip8 {
    fn font_data() -> [u8; 80] {
        [
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

    pub fn new() -> Chip8 {
        Chip8 {
            memory: vec![],
            stack: vec![],
            display: [0u64; SCREEN_HEIGHT],
            registers: [0u8; 16],
            fonts: Chip8::font_data(),
            vi: 0u16,
            pc: 0u16,
            did_jump: false,
            should_draw: false,
        }
    }

    pub fn read_rom(&mut self, path: &String) {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        self.memory.append(&mut data); // loads rom to ram
    }

    // might change this later
    fn get_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16)
    }

    fn run_opcode(&mut self, opcode: u16) {
        // for single nibble determinant
        let mut mask = 0xF000;

        match opcode & mask {
            0x1000 => self.jump(opcode),
            0x2000 => println!("CALL addr"),
            0x3000 => println!("SE Vx, byte"),
            0x4000 => println!("SNE Vx, byte"),
            0x5000 => println!("SE Vx, Vy"),
            0x6000 => self.load_register(opcode),
            0x7000 => self.add_to_register(opcode),
            0x9000 => println!("SNE Vx, Vy"),
            0xA000 => self.set_vi(opcode),
            0xB000 => println!("JP V0, addr"),
            0xC000 => println!("RND Vx, byte"),
            0xD000 => self.draw(opcode),
            _ => {}
        }

        // for dual nibble determinant
        mask = 0xF00F;

        match opcode & mask {
            0x0000 => self.clear_screen(),
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

    pub fn step(&mut self) {
        let opcode = self.get_opcode();

        println!("{:#06x}", opcode);
        self.run_opcode(opcode);

        if !self.did_jump {
            self.pc += 2;
        };

        self.did_jump = false;
    }

    /* opcode functions (might change) */
    fn clear_screen(&mut self) {
        self.display = [0u64; SCREEN_HEIGHT];
    }

    fn jump(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) - MEM_OFFSET;
        self.did_jump = true;
    }

    fn load_register(&mut self, opcode: u16) {
        let index = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;
        self.registers[index as usize] = value as u8;
    }

    fn add_to_register(&mut self, opcode: u16) {
        let index = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;

        let sum_overflow = value as u16 + self.registers[index as usize] as u16;

        self.registers[index as usize] = sum_overflow as u8; // need to check doc for this overflow
    }

    fn set_vi(&mut self, opcode: u16) {
        self.vi = opcode & 0x0FFF;
    }

    fn draw(&mut self, opcode: u16) {
        self.should_draw = true;

        let vx = ((opcode & 0x0F00) >> 8) as usize;
        let vy = ((opcode & 0x00F0) >> 4) as usize;

        let x = self.registers[vx] & (SCREEN_WIDTH - 1);
        let y = (self.registers[vy] & SCREEN_HEIGHT as u8 - 1) as usize;

        let n = (opcode & 0x000F) as usize;

        for (index, line) in self.display[y..n + y].iter_mut().enumerate() {
            let address = self.vi + index as u16;

            let sprite = (if address >= MEM_OFFSET {
                self.memory[(address - MEM_OFFSET) as usize]
            } else {
                self.fonts[(address - FONT_OFFSET) as usize]
            }) as u64;

            let offset_sprite = sprite << (SCREEN_WIDTH - 8) >> x; // Fixes subtract overflow

            let new_line = *line ^ offset_sprite;

            /* Set VF to 1 if flips bit to off */
            if new_line < *line {
                self.registers[0xF] = 1;
            }

            *line = new_line;
        }
    }
}
