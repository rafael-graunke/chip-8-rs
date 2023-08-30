use rand::Rng;
use sdl2::audio::{AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::{EventPump, Sdl};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use crate::audio::SquareWave;
use crate::screen::{Display, SCREEN_HEIGHT, SCREEN_WIDTH};

const MEM_OFFSET: u16 = 0x200;
const FONT_OFFSET: u16 = 0x50;
const DEBUG: bool = true;

struct Quirks {
    increment_index: bool,
    shifting: bool,
    jumping: bool,
}

pub struct Chip8 {
    memory: Vec<u8>,
    stack: Vec<u16>,
    display: Display,
    registers: [u8; 16],
    opcode: u16,
    vi: u16,
    pc: u16,
    did_jump: bool,
    should_wait: bool,
    delay_timer: u8,
    sound_timer: u8,
    sound_device: AudioDevice<SquareWave>,
    event_pump: EventPump,
    quirks: Quirks,
    should_draw: bool,
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PC: {:#06x}\nVI: {:#06x}\nOPCODE: {:#06x}\nREGISTERS: {:?}\nSTACK: {:?}\n",
            self.pc, self.vi, self.opcode, self.registers, self.stack
        )
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        // Initialize memory
        let mut memory = vec![0u8; 4096];

        // Iterate over fonts and add corresponding byte to address in memory
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

        for (index, byte) in font_data.iter().enumerate() {
            let address = index + FONT_OFFSET as usize;
            memory[address] = *byte;
        }

        // Initialize SDL2 and Event Pump
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        // Initialize SDL2 Audio Subsystem
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                SquareWave::new(440.0 / spec.freq as f32, 0.0, 0.05)
            })
            .unwrap();

        // Return new instance
        Chip8 {
            memory: memory,
            stack: vec![],
            display: Display::new(&sdl_context),
            registers: [0u8; 16],
            opcode: 0u16,
            vi: 0u16,
            pc: MEM_OFFSET,
            delay_timer: 0u8,
            sound_timer: 0u8,
            sound_device: device,
            did_jump: false,
            should_wait: false,
            event_pump: event_pump,
            quirks: Quirks {
                increment_index: true,
                shifting: true,
                jumping: true,
            },
            should_draw: false,
        }
    }

    pub fn read_rom(&mut self, path: &String) {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        for (index, byte) in data.iter_mut().enumerate() {
            let address = index + MEM_OFFSET as usize;
            self.memory[address] = *byte;
        }

        self.memory.append(&mut data); // loads rom to ram
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
            0xB000 => self.jump_with_offset(),
            0xC000 => self.random_number(),
            0xD000 => self.draw(),
            _ => {}
        }

        // for dual nibble determinant
        mask = 0xF00F;

        match self.opcode & mask {
            0x0000 => self.clear_screen(),
            0x000E => self.return_subroutine(),
            0xE00E => self.skip_if_key(),
            0xE001 => self.skip_if_not_key(),
            _ => {}
        }

        //for F codes
        mask = 0xF0FF;

        match self.opcode & mask {
            0xF007 => self.load_from_dt(),
            0xF00A => self.wait_for_input(),
            0xF015 => self.load_to_dt(),
            0xF018 => self.load_to_st(),
            0xF01E => self.add_to_index(),
            0xF029 => self.set_font_character(),
            0xF033 => self.binary_coded_decimal(),
            0xF055 => self.load_to_memory(),
            0xF065 => self.load_from_memory(),
            _ => {}
        }
    }

    pub fn step(&mut self, ipf: u32) -> bool {
        let mut running = true;

        if DEBUG {
            println!("{:?}", self);
        }

        if !self.should_wait {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => running = false,
                    _ => {}
                }
            }
        }

        for _ in 0..ipf {
            self.update_opcode();
            self.run_opcode();

            if !self.did_jump && !self.should_wait {
                self.pc += 2;
            };

            self.did_jump = false;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_device.resume();
            self.sound_timer -= 1;
        } else {
            self.sound_device.pause();
        }

        running
    }

    fn jump_with_offset(&mut self) {
        let mut vx = (self.opcode & 0x0F00) >> 8;

        if self.quirks.jumping {
            vx = 0;
        }

        let address = (self.opcode & 0x0FFF) + self.registers[vx as usize] as u16;

        self.pc = address;
        self.did_jump = true;
    }

    fn add_to_index(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.vi += self.registers[vx as usize] as u16;
    }

    fn key_map(key: u8) -> Scancode {
        let key_mapping = HashMap::from([
            (0x1, Scancode::Num1),
            (0x2, Scancode::Num2),
            (0x3, Scancode::Num3),
            (0xC, Scancode::Num4),
            (0x4, Scancode::Q),
            (0x5, Scancode::W),
            (0x6, Scancode::E),
            (0xD, Scancode::R),
            (0x7, Scancode::A),
            (0x8, Scancode::S),
            (0x9, Scancode::D),
            (0xE, Scancode::F),
            (0xA, Scancode::Z),
            (0x0, Scancode::X),
            (0xB, Scancode::C),
            (0xF, Scancode::V),
        ]);

        *key_mapping.get(&key).unwrap()
    }

    fn skip_if_key(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        let key = Chip8::key_map(self.registers[vx as usize]);

        if self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .any(|x| x == key)
        {
            self.pc += 2;
        };
    }

    fn skip_if_not_key(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        let key = Chip8::key_map(self.registers[vx as usize]);

        if self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .all(|x| x != key)
        {
            self.pc += 2;
        };
    }

    fn wait_for_input(&mut self) {
        self.should_wait = true;
        let vx = (self.opcode & 0x0F00) >> 8;

        let key_mapping = HashMap::from([
            (Keycode::Num1, 0x1),
            (Keycode::Num2, 0x2),
            (Keycode::Num3, 0x3),
            (Keycode::Num4, 0xC),
            (Keycode::Q, 0x4),
            (Keycode::W, 0x5),
            (Keycode::E, 0x6),
            (Keycode::R, 0xD),
            (Keycode::A, 0x7),
            (Keycode::S, 0x8),
            (Keycode::D, 0x9),
            (Keycode::F, 0xE),
            (Keycode::Z, 0xA),
            (Keycode::X, 0x0),
            (Keycode::C, 0xB),
            (Keycode::V, 0xF),
        ]);

        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match key_mapping.get(&keycode) {
                    Some(key) => {
                        self.registers[vx as usize] = *key;
                        self.should_wait = false;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn load_from_dt(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.registers[vx as usize] = self.delay_timer;
    }

    fn load_to_dt(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.delay_timer = self.registers[vx as usize];
    }

    fn load_to_st(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.sound_timer = self.registers[vx as usize];
    }

    fn set_font_character(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let x = self.registers[vx as usize];
        self.vi = FONT_OFFSET + (x * 5) as u16;
    }

    fn load_to_memory(&mut self) {
        let x = (self.opcode & 0x0F00) >> 8;

        for i in 0..=x {
            let mem_address = self.vi + i;
            self.memory[mem_address as usize] = self.registers[i as usize];
        }

        if self.quirks.increment_index {
            self.vi += x + 1;
        }
    }

    fn load_from_memory(&mut self) {
        let x: u16 = (self.opcode & 0x0F00) >> 8;

        for i in 0..=x {
            let mem_address = (self.vi) + i;
            self.registers[i as usize] = self.memory[mem_address as usize];
        }
    }

    fn binary_coded_decimal(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let x = self.registers[vx as usize];

        let address = self.vi;

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

        self.registers[0xF] = 0;
        self.registers[vx] = match operation {
            0 => y,
            1 => x | y,
            2 => x & y,
            3 => x ^ y,
            4 => self.sum_overflow(x, y),
            5 => self.subtract_overflow(x, y),
            6 => {
                self.registers[0xF] = x & 1;
                if self.quirks.shifting {
                    y >> 1
                } else {
                    x >> 1
                }
            }
            7 => self.subtract_overflow(y, x),
            0xE => {
                self.registers[0xF] = (x & 0x80) >> 7;
                if self.quirks.shifting {
                    y << 1
                } else {
                    x << 1
                }
            }
            _ => 0,
        }
    }

    fn sum_overflow(&mut self, n1: u8, n2: u8) -> u8 {
        let n1 = n1 as u16;
        let n2 = n2 as u16;

        let sum = n1 + n2;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        return sum as u8;
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
        self.pc = self.opcode & 0x0FFF;
        self.did_jump = true;
    }

    fn return_subroutine(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn clear_screen(&mut self) {
        self.display.clear();
    }

    fn jump(&mut self) {
        self.pc = self.opcode & 0x0FFF;
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

        // This overflow does not affect F flag
        let mut sum = value + self.registers[index as usize] as u16;

        if sum > 255 {
            sum -= 256;
        }

        self.registers[index as usize] = sum as u8;
    }

    fn set_vi(&mut self) {
        self.vi = self.opcode & 0x0FFF;
    }

    fn draw(&mut self) {
        self.should_draw = true;

        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        let x = self.registers[vx] & (SCREEN_WIDTH - 1);
        let y = (self.registers[vy] & (SCREEN_HEIGHT as u8 - 1)) as usize;

        let n = (self.opcode & 0x000F) as usize;

        self.registers[0xF] = 0;

        for index in 0..n {
            let line =
                &mut self.display.screen_memory[(y + index) & (SCREEN_HEIGHT as u8 - 1) as usize];

            let address = self.vi + index as u16;

            let sprite = self.memory[address as usize] as u64;

            let offset_sprite = sprite << (SCREEN_WIDTH - 8) >> x;

            let new_line = *line ^ offset_sprite;

            for bit in 0..SCREEN_WIDTH {
                let bit_before = *line & (0x8000000000000000 >> bit);
                let bit_after = new_line & (0x8000000000000000 >> bit);

                if (bit_before != bit_after) && (bit_before >> (SCREEN_WIDTH - 1 - bit)) == 1 {
                    self.registers[0xF] = 1;
                }
            }

            *line = new_line;
        }
    }
}
