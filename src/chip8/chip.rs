use sdl2::audio::{AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::fs::File;
use std::io::prelude::*;

use crate::audio::SquareWave;
use crate::screen::Screen;

use crate::chip8::opcodes::handlers;
use crate::chip8::quirks::Quirks;
use crate::chip8::state::ChipState;

const MEM_OFFSET: u16 = 0x200;
const DEBUG: bool = false;

pub struct Chip8 {
    state: ChipState,
    display: Screen,
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

    pub fn fetch(&self) -> u16 {
        let addr = self.state.pc as usize;
        let bytes = <[u8; 2]>::try_from(&self.state.memory[addr..=addr + 1]).unwrap();
        u16::from_be_bytes(bytes)
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

        // let mut wait_for_step = true;

        // while wait_for_step {
        //     for event in self.event_pump.poll_iter() {
        //         match event {
        //             Event::Quit { .. }
        //             | Event::KeyDown {
        //                 keycode: Some(Keycode::Escape),
        //                 ..
        //             } => self.state.running = false,
        //             Event::KeyDown {
        //                 keycode: Some(Keycode::Space),
        //                 ..
        //             } => {
        //                 println!("{:?}", self.state);
        //                 wait_for_step = false;
        //             }
        //             _ => {}
        //         }
        //     }
        // }

        // Run N instructions per seconds
        for _ in 0..ipf {
            let code = self.fetch();

            handlers::run(
                code,
                &mut self.state,
                &self.quirks,
                &mut self.display,
                &mut self.event_pump,
            );

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
}
