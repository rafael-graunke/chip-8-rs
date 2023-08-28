use rand::Rng;
use sdl2::Sdl;
use std::fs::File;
use std::io::prelude::*;

use crate::screen::{Display, SCREEN_HEIGHT, SCREEN_WIDTH};

const MEM_OFFSET: u16 = 0x200;
const FONT_OFFSET: u16 = 0x50;

pub struct Chip8 {
    memory: Vec<u8>,
    stack: Vec<u16>,
    display: Display,
    registers: [u8; 16],
    fonts: [u8; 80],
    opcode: u16,
    vi: u16,
    pc: u16,
    did_jump: bool,
    delay_timer: u8,
    pub should_draw: bool,
}

impl Chip8 {
    pub fn new(sdl: &Sdl) -> Chip8 {
        let font_data = [
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
        ];

        Chip8 {
            memory: vec![],
            stack: vec![],
            display: Display::new(&sdl),
            registers: [0u8; 16],
            fonts: font_data,
            opcode: 0u16,
            vi: 0u16,
            pc: 0u16,
            delay_timer: 0u8,
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

    pub fn get_input(&self) {
        unimplemented!();
    }

    pub fn render(&mut self) {
        self.display.render();
    }

    fn update_opcode(&mut self) {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
    }

    fn run_opcode(&mut self) {
        // for single nibble determinant
        let mut mask = 0xF000;

        match self.opcode & mask {
            0x1000 => self.jump(),
            0x2000 => self.call_subroutine(),
            0x3000 => self.skip_equal(),
            0x4000 => self.skip_not_equal(),
            0x5000 => self.skip_register_equal(),
            0x6000 => self.load_register(),
            0x7000 => self.add_to_register(),
            0x8000 => self.logic_op(),
            0x9000 => self.skip_register_not_equal(),
            0xA000 => self.set_vi(),
            0xB000 => println!("JP V0, addr"), // make it configurable
            0xC000 => self.random_number(),
            0xD000 => self.draw(),
            _ => {}
        }

        // for dual nibble determinant
        mask = 0xF00F;

        match self.opcode & mask {
            0x0000 => self.clear_screen(),
            0x000E => self.return_subroutine(),
            0xE00E => println!("SKP Vx"),
            0xE001 => println!("SKNP Vx1"),
            _ => {}
        }

        //for F codes
        mask = 0xF0FF;

        match self.opcode & mask {
            0xF007 => self.load_from_dt(),
            0xF00A => println!("LD Vx, K"),
            0xF015 => self.load_to_dt(),
            0xF018 => println!("LD ST, Vx"),
            0xF01E => println!("ADD I, Vx"),
            0xF029 => self.set_font_character(),
            0xF033 => self.binary_coded_decimal(),
            0xF055 => self.load_to_memory(),
            0xF065 => self.load_from_memory(),
            _ => {}
        }
    }

    pub fn step(&mut self, ipf: u32) {
        for _ in 0..ipf {
            self.update_opcode();
            self.run_opcode();

            if !self.did_jump {
                self.pc += 2;
            };

            self.did_jump = false;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    /* opcode functions (might change(?)) */
    fn load_from_dt(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.registers[vx as usize] = self.delay_timer;
    }

    fn load_to_dt(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.delay_timer = self.registers[vx as usize];
    }

    fn set_font_character(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let x = self.registers[vx as usize];

        self.vi = (x * 5) as u16;
    }

    fn load_to_memory(&mut self) {
        let x = (self.opcode & 0x0F00) >> 8;

        for i in 0..=x {
            let mem_address = (self.vi - MEM_OFFSET) + i;
            self.memory[mem_address as usize] = self.registers[i as usize];
        }
    }

    fn load_from_memory(&mut self) {
        let x: u16 = (self.opcode & 0x0F00) >> 8;

        for i in 0..=x {
            let mem_address = (self.vi - MEM_OFFSET) + i;
            self.registers[i as usize] = self.memory[mem_address as usize];
        }
    }

    fn binary_coded_decimal(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let x = self.registers[vx as usize];

        let address = self.vi - MEM_OFFSET;

        self.memory[(address) as usize] = x / 100;
        self.memory[(address + 1) as usize] = (x % 100) / 10;
        self.memory[(address + 2) as usize] = x % 10;
    }

    fn logic_op(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let x = self.registers[vx];

        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let y = self.registers[vy];

        let operation = self.opcode & 0x000F;

        self.registers[vx] = match operation {
            0 => y,
            1 => x | y,
            2 => x & y,
            3 => x ^ y,
            4 => (x as u16 + y as u16) as u8,
            5 => self.subtract_overflow(x, y),
            7 => self.subtract_overflow(y, x),
            _ => 0,
        }
    }

    fn subtract_overflow(&mut self, n1: u8, n2: u8) -> u8 {
        let mut n1 = n1 as u16;
        let n2 = n2 as u16;

        if n1 >= n2 {
            self.registers[0xF] = 1;
            (n1 - n2) as u8
        } else {
            self.registers[0xF] = 0;
            n1 |= 0x100;
            (n1 - n2) as u8
        }
    }

    fn random_number(&mut self) {
        let mut rng = rand::thread_rng();

        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        let nn: u8 = (self.opcode & 0x00FF) as u8;
        let random: u8 = rng.gen();

        self.registers[vx] = random & nn;
    }

    fn skip_equal(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let x = self.registers[vx];

        let value = (self.opcode & 0x00FF) as u8;
        if x == value {
            self.pc += 2;
        }
    }

    fn skip_not_equal(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let x = self.registers[vx];

        let value = (self.opcode & 0x00FF) as u8;
        if x != value {
            self.pc += 2;
        }
    }

    fn skip_register_equal(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let x = self.registers[vx];

        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let y = self.registers[vy];

        if x == y {
            self.pc += 2;
        }
    }

    fn skip_register_not_equal(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let x = self.registers[vx];

        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let y = self.registers[vy];

        if x != y {
            self.pc += 2;
        }
    }

    fn call_subroutine(&mut self) {
        self.stack.push(self.pc);
        self.pc = (self.opcode & 0x0FFF) - MEM_OFFSET;
        self.did_jump = true;
    }

    fn return_subroutine(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn clear_screen(&mut self) {
        self.display.clear();
    }

    fn jump(&mut self) {
        self.pc = (self.opcode & 0x0FFF) - MEM_OFFSET;
        self.did_jump = true;
    }

    fn load_register(&mut self) {
        let index = (self.opcode & 0x0F00) >> 8;
        let value = self.opcode & 0x00FF;
        self.registers[index as usize] = value as u8;
    }

    fn add_to_register(&mut self) {
        let index = (self.opcode & 0x0F00) >> 8;
        let value = self.opcode & 0x00FF;

        let sum_overflow = value as u16 + self.registers[index as usize] as u16;

        self.registers[index as usize] = sum_overflow as u8; // need to check doc for this overflow
    }

    fn set_vi(&mut self) {
        self.vi = self.opcode & 0x0FFF;
    }

    fn draw(&mut self) {
        self.should_draw = true;

        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        let x = self.registers[vx] & (SCREEN_WIDTH - 1);
        let y = (self.registers[vy] & SCREEN_HEIGHT as u8 - 1) as usize;

        let n = (self.opcode & 0x000F) as usize;

        for (index, line) in self.display.screen_memory[y..n + y].iter_mut().enumerate() {
            let address = self.vi + index as u16;

            let sprite = (if address >= MEM_OFFSET {
                self.memory[(address - MEM_OFFSET) as usize]
            } else {
                self.fonts[(address) as usize]
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
