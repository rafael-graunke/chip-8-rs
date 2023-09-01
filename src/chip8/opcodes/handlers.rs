use std::collections::HashMap;

use crate::chip8::opcodes::parser;
use crate::chip8::quirks::Quirks;
use crate::chip8::state::ChipState;
use crate::screen::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;

pub fn run(
    opcode: u16,
    state: &mut ChipState,
    quirks: &Quirks,
    screen: &mut Screen,
    events: &mut EventPump,
) {
    match parser::get_digit(opcode) {
        0x0 => run_00en(opcode, state, screen),
        0x1 => run_1nnn(opcode, state),
        0x2 => run_2nnn(opcode, state),
        0x3 => run_3xnn(opcode, state),
        0x4 => run_4xnn(opcode, state),
        0x5 => run_5xy0(opcode, state),
        0x6 => run_6xnn(opcode, state),
        0x7 => run_7xnn(opcode, state),
        0x8 => run_8xyn(opcode, state, quirks.has_shifting()),
        0x9 => run_9xy0(opcode, state),
        0xA => run_annn(opcode, state),
        0xB => run_bnnn(opcode, state, !quirks.has_jumping()),
        0xC => run_cxnn(opcode, state),
        0xD => run_dxyn(opcode, state, screen),
        0xE => run_exnn(opcode, state, events),
        0xF => run_fxnn(opcode, state, events),
        _ => {}
    }
}

pub fn run_00en(opcode: u16, state: &mut ChipState, screen: &mut Screen) {
    match parser::get_n(opcode) {
        0x0 => screen.clear(),
        0xE => state.pc = state.stack.pop().unwrap(),
        _ => {}
    }
}

pub fn run_1nnn(opcode: u16, state: &mut ChipState) {
    state.pc = parser::get_3n(opcode);
    state.did_jump = true;
}

pub fn run_2nnn(opcode: u16, state: &mut ChipState) {
    state.stack.push(state.pc);
    state.pc = parser::get_3n(opcode);
    state.did_jump = true;
}

pub fn run_3xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    if vx == parser::get_2n(opcode) {
        state.skip();
    }
}

pub fn run_4xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    if vx != parser::get_2n(opcode) {
        state.skip();
    }
}

pub fn run_5xy0(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    let y = parser::get_y(opcode) as usize;
    let vy = state.registers[y];

    if vx == vy {
        state.skip();
    }
}

pub fn run_6xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;

    state.registers[x] = parser::get_2n(opcode);
}

pub fn run_7xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;

    let sum = state.registers[x] as u16 + parser::get_2n(opcode) as u16;

    state.registers[x] = sum as u8;
}

pub fn run_8xyn(opcode: u16, state: &mut ChipState, change_x: bool) {
    let x = parser::get_x(opcode) as usize;
    let y = parser::get_y(opcode) as usize;
    let op = parser::get_n(opcode);

    let vx = state.registers[x];
    let vy = state.registers[y];

    let mut carry = 0u8;

    state.registers[x] = match op {
        0x0 => vy,
        0x1 => vx | vy,
        0x2 => vx & vy,
        0x3 => vx ^ vy,
        0x4 => {
            let sum = (vx as u16) + (vy as u16);
            if sum > 255 {
                carry = 1; // Set carry flag
            }
            sum as u8
        }
        0x5 => {
            if vx < vy {
                ((vx as u16 | 0x100) - (vy as u16)) as u8
            } else {
                carry = 1;
                ((vx as u16) - (vy as u16)) as u8
            }
        }
        0x6 => {
            if change_x {
                carry = vy & 1;
                vy >> 1
            } else {
                carry = vx & 1;
                vx >> 1
            }
        }
        0x7 => {
            if vy < vx {
                ((vy as u16 | 0x100) - (vx as u16)) as u8
            } else {
                carry = 1;
                ((vy as u16) - (vx as u16)) as u8
            }
        }
        0xE => {
            if change_x {
                carry = (vy & 0x80) >> 7;
                vy << 1
            } else {
                carry = (vx & 0x80) >> 7;
                vx << 1
            }
        }
        _ => 0,
    };

    state.registers[15] = carry;
}

pub fn run_9xy0(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    let y = parser::get_y(opcode) as usize;
    let vy = state.registers[y];

    if vx != vy {
        state.skip();
    }
}

pub fn run_annn(opcode: u16, state: &mut ChipState) {
    state.vi = parser::get_3n(opcode);
}

pub fn run_bnnn(opcode: u16, state: &mut ChipState, use_x: bool) {
    let x = if use_x { parser::get_x(opcode) } else { 0 };

    let address = parser::get_3n(opcode) + state.registers[x as usize] as u16;

    state.pc = address;
    state.did_jump = true;
}

pub fn run_cxnn(opcode: u16, state: &mut ChipState) {
    let mut rng = rand::thread_rng();

    let x = parser::get_x(opcode) as usize;

    let random: u8 = rng.gen();

    state.registers[x] = random & parser::get_2n(opcode);
}

pub fn run_dxyn(opcode: u16, state: &mut ChipState, screen: &mut Screen) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x] & (SCREEN_WIDTH - 1);

    let y = parser::get_y(opcode) as usize;
    let vy = state.registers[y] & (SCREEN_HEIGHT as u8 - 1);

    let n = parser::get_n(opcode);

    state.registers[15] = 0;

    for index in 0..n {
        if index + vy <= 31 {
            let wrap_pos = (vy + index) & (SCREEN_HEIGHT - 1) as u8;
            let line = &mut screen.screen_memory[wrap_pos as usize];

            let address = state.vi + index as u16;

            let sprite = state.memory[address as usize] as u64;

            let offset_sprite = sprite << (SCREEN_WIDTH - 8) >> vx;

            let new_line = *line ^ offset_sprite;

            for bit in 0..SCREEN_WIDTH {
                let bit_before = *line & (0x8000000000000000 >> bit);
                let bit_after = new_line & (0x8000000000000000 >> bit);

                if (bit_before != bit_after) && (bit_before >> (SCREEN_WIDTH - 1 - bit)) == 1 {
                    state.registers[15] = 1;
                }
            }

            *line = new_line;
        }
    }

    state.should_draw = true;
}

pub fn run_exnn(opcode: u16, state: &mut ChipState, events: &mut EventPump) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    let key = u8_to_scancode(vx);

    match parser::get_2n(opcode) {
        0x9E => {
            if events
                .keyboard_state()
                .pressed_scancodes()
                .any(|code| code == key)
            {
                state.skip()
            }
        }
        0xA1 => {
            if events
                .keyboard_state()
                .pressed_scancodes()
                .all(|code| code != key)
            {
                state.skip()
            }
        }
        _ => {}
    }
}

pub fn run_fxnn(opcode: u16, state: &mut ChipState, events: &mut EventPump) {
    fn wait_for_key(x: usize, state: &mut ChipState, events: &mut EventPump) {
        state.should_wait = true;

        for event in events.poll_iter() {
            match event {
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode_to_u8(keycode) {
                    Some(key) => {
                        state.registers[x as usize] = key;
                        state.should_wait = false;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn load_to_memory(x: u16, state: &mut ChipState, update_vi: bool) {
        for i in 0..=x {
            let mem_address = state.vi + i;
            state.memory[mem_address as usize] = state.registers[i as usize];
        }

        if update_vi {
            state.vi += x + 1;
        }
    }

    fn load_from_memory(x: u16, state: &mut ChipState) {
        for i in 0..=x {
            let addr = state.vi + i;
            state.registers[i as usize] = state.memory[addr as usize];
        }
    }

    fn binary_coded_decimal(vx: u8, state: &mut ChipState) {
        let address = state.vi;

        state.memory[(address) as usize] = vx / 100;
        state.memory[(address + 1) as usize] = (vx % 100) / 10;
        state.memory[(address + 2) as usize] = vx % 10;
    }

    let x = parser::get_x(opcode) as usize;
    let vx = &mut state.registers[x];

    let op = parser::get_2n(opcode);

    match op {
        0x07 => *vx = state.delay_timer,
        0x0A => wait_for_key(x, state, events),
        0x15 => state.delay_timer = *vx,
        0x18 => state.sound_timer = *vx,
        0x1E => state.vi += *vx as u16,
        0x29 => state.vi = 0x50 + (*vx * 5) as u16,
        0x33 => binary_coded_decimal(*vx, state),
        0x55 => load_to_memory(x as u16, state, true),
        0x65 => load_from_memory(x as u16, state),
        _ => {}
    }
}

fn u8_to_scancode(key: u8) -> Scancode {
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

fn keycode_to_u8(key: Keycode) -> Option<u8> {
    let key_mapping: HashMap<Keycode, u8> = HashMap::from([
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

    key_mapping.get(&key).copied()
}
