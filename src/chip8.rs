use rand::Rng;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::sys::{KeyCode, KeyPress};
use sdl2::{EventPump, Sdl};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use crate::screen::{Display, SCREEN_HEIGHT, SCREEN_WIDTH};

const MEM_OFFSET: u16 = 0x200;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Chip8<'a> {
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
    sound_timer: u8,
    sound_device: AudioDevice<SquareWave>,
    event_pump: &'a mut EventPump,
    pub should_draw: bool,
}

impl Chip8<'_> {
    pub fn new<'a>(sdl: &'a Sdl, event_pump: &'a mut EventPump) -> Chip8<'a> {
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

        let audio_subsystem = sdl.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.05,
                }
            })
            .unwrap();

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
            sound_timer: 0u8,
            sound_device: device,
            did_jump: false,
            event_pump: event_pump,
            should_draw: false,
        }
    }

    pub fn read_rom(&mut self, path: &String) {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

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

        if self.sound_timer > 0 {
            self.sound_device.resume();
            self.sound_timer -= 1;
        } else {
            self.sound_device.pause();
        }

        running
    }

    fn add_to_index(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.vi += self.registers[vx as usize] as u16;
    }

    fn skip_if_key(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        let key_mapping = HashMap::from([
            (0, Scancode::Num1),
            (1, Scancode::Num2),
            (2, Scancode::Num3),
            (3, Scancode::Num4),
            (4, Scancode::Q),
            (5, Scancode::W),
            (6, Scancode::E),
            (7, Scancode::R),
            (8, Scancode::A),
            (9, Scancode::S),
            (10, Scancode::D),
            (11, Scancode::F),
            (12, Scancode::Z),
            (13, Scancode::X),
            (14, Scancode::C),
            (15, Scancode::V),
        ]);

        let key = key_mapping.get(&self.registers[vx as usize]).unwrap();

        if self.event_pump.keyboard_state().pressed_scancodes().any(|x| x == *key) {
            self.pc += 2;
        };
    }

    fn skip_if_not_key(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        let key_mapping = HashMap::from([
            (0, Scancode::Num1),
            (1, Scancode::Num2),
            (2, Scancode::Num3),
            (3, Scancode::Num4),
            (4, Scancode::Q),
            (5, Scancode::W),
            (6, Scancode::E),
            (7, Scancode::R),
            (8, Scancode::A),
            (9, Scancode::S),
            (10, Scancode::D),
            (11, Scancode::F),
            (12, Scancode::Z),
            (13, Scancode::X),
            (14, Scancode::C),
            (15, Scancode::V),
        ]);

        let key = key_mapping.get(&self.registers[vx as usize]).unwrap();

        if self.event_pump.keyboard_state().pressed_scancodes().all(|x| x != *key) {
            self.pc += 2;
        };
    }

    fn wait_for_input(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        let mut key_pressed = false;

        while !key_pressed {
            for event in self.event_pump.poll_iter() {
                let key = match event {
                    Event::KeyUp {
                        keycode: Some(Keycode::Num1),
                        ..
                    } => Some(0),
                    Event::KeyUp {
                        keycode: Some(Keycode::Num2),
                        ..
                    } => Some(1),
                    Event::KeyUp {
                        keycode: Some(Keycode::Num3),
                        ..
                    } => Some(2),
                    Event::KeyUp {
                        keycode: Some(Keycode::Num4),
                        ..
                    } => Some(3),
                    Event::KeyUp {
                        keycode: Some(Keycode::Q),
                        ..
                    } => Some(4),
                    Event::KeyUp {
                        keycode: Some(Keycode::W),
                        ..
                    } => Some(5),
                    Event::KeyUp {
                        keycode: Some(Keycode::E),
                        ..
                    } => Some(6),
                    Event::KeyUp {
                        keycode: Some(Keycode::R),
                        ..
                    } => Some(7),
                    Event::KeyUp {
                        keycode: Some(Keycode::A),
                        ..
                    } => Some(8),
                    Event::KeyUp {
                        keycode: Some(Keycode::S),
                        ..
                    } => Some(9),
                    Event::KeyUp {
                        keycode: Some(Keycode::D),
                        ..
                    } => Some(10),
                    Event::KeyUp {
                        keycode: Some(Keycode::F),
                        ..
                    } => Some(11),
                    Event::KeyUp {
                        keycode: Some(Keycode::Z),
                        ..
                    } => Some(12),
                    Event::KeyUp {
                        keycode: Some(Keycode::X),
                        ..
                    } => Some(13),
                    Event::KeyUp {
                        keycode: Some(Keycode::C),
                        ..
                    } => Some(14),
                    Event::KeyUp {
                        keycode: Some(Keycode::V),
                        ..
                    } => Some(15),
                    _ => None,
                };

                match key {
                    Some(n) => {
                        self.registers[vx as usize] = n as u8;
                        key_pressed = true;
                    }
                    None => {}
                }
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
        let y = (self.registers[vy] & (SCREEN_HEIGHT as u8 - 1)) as usize;

        let n = (self.opcode & 0x000F) as usize;

        for (index, line) in self.display.screen_memory[y..(n + y)].iter_mut().enumerate() {
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
