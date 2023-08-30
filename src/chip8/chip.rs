use rand::Rng;
use sdl2::audio::{AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use crate::audio::SquareWave;
use crate::screen::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};

use crate::chip8::opcode::OpCode;
use crate::chip8::quirks::Quirks;
use crate::chip8::state::ChipState;

const MEM_OFFSET: u16 = 0x200;
const FONT_OFFSET: u16 = 0x50;
const DEBUG: bool = true;

pub struct Chip8 {
    state: ChipState,
    display: Screen,
    opcode: OpCode,
    sound_device: AudioDevice<SquareWave>,
    event_pump: EventPump,
    quirks: Quirks,
}

impl Chip8 {
    pub fn new() -> Chip8 {
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
            state: ChipState::init(),
            display: Screen::new(&sdl_context),
            opcode: OpCode::init(),
            sound_device: device,
            event_pump: event_pump,
            quirks: Quirks::for_chip8(),
        }
    }

    pub fn read_rom(&mut self, path: &String) {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        for (index, byte) in data.iter_mut().enumerate() {
            let address = index + MEM_OFFSET as usize;
            self.state.memory[address] = *byte;
        }
    }

    pub fn render(&mut self) {
        if self.state.should_draw {
            self.display.render();
            self.state.should_draw = false;
        };
    }

    pub fn is_running(&self) -> bool {
        self.state.running
    }

    fn fetch_opcode(&mut self) {
        let first_byte = self.state.memory[self.state.pc as usize] as u16;
        let second_byte = self.state.memory[(self.state.pc + 1) as usize] as u16;

        let new_code = (first_byte << 8) | second_byte;

        self.opcode.set(new_code);
    }

    fn run_opcode(&mut self) {
        // Check single nibble determinant opcodes
        match self.opcode.mask(0xF000) {
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

        // Check dual nibble determinant opcodes
        match self.opcode.mask(0xF00F) {
            0x0000 => self.clear_screen(),
            0x000E => self.return_subroutine(),
            0xE00E => self.skip_if_key(),
            0xE001 => self.skip_if_not_key(),
            _ => {}
        }

        // Check F opcodes
        match self.opcode.mask(0xF0FF) {
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

    pub fn step(&mut self, ipf: u32) {
        // If not waiting for input key
        if !self.state.should_wait {
            // Check events, if exit then set running to false
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => self.state.running = false,
                    _ => {}
                }
            }
        }

        // Run N instructions per seconds
        for _ in 0..ipf {
            self.fetch_opcode();
            self.run_opcode();

            if !self.state.did_jump && !self.state.should_wait {
                self.state.pc += 2;
            };

            self.state.did_jump = false;
        }

        // Decrease delay timer
        if self.state.delay_timer > 0 {
            self.state.delay_timer -= 1;
        }

        // Decrese sound timer and play sound until reach zero
        if self.state.sound_timer > 0 {
            self.sound_device.resume();
            self.state.sound_timer -= 1;
        } else {
            self.sound_device.pause();
        }

        // Render frame
        self.render();

        if DEBUG {
            println!("{:?}", self.state);
        };
    }

    fn jump_with_offset(&mut self) {
        let mut vx = self.opcode.get_x();

        if self.quirks.has_jumping() {
            vx = 0;
        }

        let address = self.opcode.get_3n() + self.state.registers[vx as usize] as u16;

        self.state.pc = address;
        self.state.did_jump = true;
    }

    fn add_to_index(&mut self) {
        let vx = self.opcode.get_x();
        self.state.vi += self.state.registers[vx as usize] as u16;
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
        let vx = self.opcode.get_x();

        let key = Chip8::key_map(self.state.registers[vx as usize]);

        if self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .any(|x| x == key)
        {
            self.state.pc += 2;
        };
    }

    fn skip_if_not_key(&mut self) {
        let vx = self.opcode.get_x();

        let key = Chip8::key_map(self.state.registers[vx as usize]);

        if self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .all(|x| x != key)
        {
            self.state.pc += 2;
        };
    }

    fn wait_for_input(&mut self) {
        self.state.should_wait = true;
        let vx = self.opcode.get_x();

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
                        self.state.registers[vx as usize] = *key;
                        self.state.should_wait = false;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn load_from_dt(&mut self) {
        let vx = self.opcode.get_x();
        self.state.registers[vx as usize] = self.state.delay_timer;
    }

    fn load_to_dt(&mut self) {
        let vx = self.opcode.get_x();
        self.state.delay_timer = self.state.registers[vx as usize];
    }

    fn load_to_st(&mut self) {
        let vx = self.opcode.get_x();
        self.state.sound_timer = self.state.registers[vx as usize];
    }

    fn set_font_character(&mut self) {
        let vx = self.opcode.get_x();
        let x = self.state.registers[vx as usize];
        self.state.vi = FONT_OFFSET + (x * 5) as u16;
    }

    fn load_to_memory(&mut self) {
        let x = self.opcode.get_x() as u16;

        for i in 0..=x {
            let mem_address = self.state.vi + i;
            self.state.memory[mem_address as usize] = self.state.registers[i as usize];
        }

        if self.quirks.has_increment_index() {
            self.state.vi += x + 1;
        }
    }

    fn load_from_memory(&mut self) {
        let x = self.opcode.get_x() as u16;

        for i in 0..=x {
            let mem_address = (self.state.vi) + i;
            self.state.registers[i as usize] = self.state.memory[mem_address as usize];
        }
    }

    fn binary_coded_decimal(&mut self) {
        let vx = self.opcode.get_x();
        let x = self.state.registers[vx as usize];

        let address = self.state.vi;

        self.state.memory[(address) as usize] = x / 100;
        self.state.memory[(address + 1) as usize] = (x % 100) / 10;
        self.state.memory[(address + 2) as usize] = x % 10;
    }

    fn logic_op(&mut self) {
        let vx = self.opcode.get_x() as usize;
        let x = self.state.registers[vx];

        let vy = self.opcode.get_y();
        let y = self.state.registers[vy as usize];

        let operation = self.opcode.get_n();

        self.state.registers[0xF] = 0;
        self.state.registers[vx] = match operation {
            0 => y,
            1 => x | y,
            2 => x & y,
            3 => x ^ y,
            4 => self.sum_overflow(x, y),
            5 => self.subtract_overflow(x, y),
            6 => {
                self.state.registers[0xF] = x & 1;
                if self.quirks.has_shifting() {
                    y >> 1
                } else {
                    x >> 1
                }
            }
            7 => self.subtract_overflow(y, x),
            0xE => {
                self.state.registers[0xF] = (x & 0x80) >> 7;
                if self.quirks.has_shifting() {
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
            self.state.registers[0xF] = 1;
        } else {
            self.state.registers[0xF] = 0;
        }

        return sum as u8;
    }

    fn subtract_overflow(&mut self, n1: u8, n2: u8) -> u8 {
        let mut n1 = n1 as u16;
        let n2 = n2 as u16;

        self.state.registers[0xF] = 1;

        if n1 < n2 {
            self.state.registers[0xF] = 0;
            n1 |= 0x100;
        }

        (n1 - n2) as u8
    }

    fn random_number(&mut self) {
        let mut rng = rand::thread_rng();

        let vx = self.opcode.get_x() as usize;

        let nn: u8 = self.opcode.get_2n() as u8;
        let random: u8 = rng.gen();

        self.state.registers[vx] = random & nn;
    }

    fn skip_equal(&mut self) {
        let vx = self.opcode.get_x() as usize;
        let x = self.state.registers[vx];

        let value = self.opcode.get_2n();
        if x == value {
            self.state.pc += 2;
        }
    }

    fn skip_not_equal(&mut self) {
        let vx = self.opcode.get_x() as usize;
        let x = self.state.registers[vx];

        let value = self.opcode.get_2n();
        if x != value {
            self.state.pc += 2;
        }
    }

    fn skip_register_equal(&mut self) {
        let vx = self.opcode.get_x() as usize;
        let x = self.state.registers[vx];

        let vy = self.opcode.get_y() as usize;
        let y = self.state.registers[vy];

        if x == y {
            self.state.pc += 2;
        }
    }

    fn skip_register_not_equal(&mut self) {
        let vx = self.opcode.get_x() as usize;
        let x = self.state.registers[vx];

        let vy = self.opcode.get_y() as usize;
        let y = self.state.registers[vy];

        if x != y {
            self.state.pc += 2;
        }
    }

    fn call_subroutine(&mut self) {
        self.state.stack.push(self.state.pc);
        self.state.pc = self.opcode.get_3n();
        self.state.did_jump = true;
    }

    fn return_subroutine(&mut self) {
        self.state.pc = self.state.stack.pop().unwrap();
    }

    fn clear_screen(&mut self) {
        self.display.clear();
    }

    fn jump(&mut self) {
        self.state.pc = self.opcode.get_3n();
        self.state.did_jump = true;
    }

    fn load_register(&mut self) {
        let index = self.opcode.get_x();
        let value = self.opcode.get_2n();
        self.state.registers[index as usize] = value as u8;
    }

    fn add_to_register(&mut self) {
        let index = self.opcode.get_x();
        let value = self.opcode.get_2n();

        // This overflow does not affect F flag
        let mut sum = value as u16 + self.state.registers[index as usize] as u16;

        if sum > 255 {
            sum -= 256;
        }

        self.state.registers[index as usize] = sum as u8;
    }

    fn set_vi(&mut self) {
        self.state.vi = self.opcode.get_3n();
    }

    fn draw(&mut self) {
        self.state.should_draw = true;

        let vx = self.opcode.get_x() as usize;
        let vy = self.opcode.get_y() as usize;

        let x = self.state.registers[vx] & (SCREEN_WIDTH - 1);
        let y = (self.state.registers[vy] & (SCREEN_HEIGHT as u8 - 1)) as usize;

        let n = self.opcode.get_n() as usize;

        self.state.registers[0xF] = 0;

        for index in 0..n {
            let line =
                &mut self.display.screen_memory[(y + index) & (SCREEN_HEIGHT as u8 - 1) as usize];

            let address = self.state.vi + index as u16;

            let sprite = self.state.memory[address as usize] as u64;

            let offset_sprite = sprite << (SCREEN_WIDTH - 8) >> x;

            let new_line = *line ^ offset_sprite;

            for bit in 0..SCREEN_WIDTH {
                let bit_before = *line & (0x8000000000000000 >> bit);
                let bit_after = new_line & (0x8000000000000000 >> bit);

                if (bit_before != bit_after) && (bit_before >> (SCREEN_WIDTH - 1 - bit)) == 1 {
                    self.state.registers[0xF] = 1;
                }
            }

            *line = new_line;
        }
    }
}
