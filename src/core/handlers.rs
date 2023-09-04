use std::collections::HashMap;

use crate::core::opcode::OpCode;
use crate::core::quirks::Quirks;
use crate::core::state::ChipState;
use crate::screen::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;

pub fn decode_and_run(
    opcode: OpCode,
    state: &mut ChipState,
    quirks: &Quirks,
    screen: &mut Screen,
    events: &mut EventPump,
) {
    match opcode {
        OpCode(0, 0, 0xE, 0) => run_00e0(screen),
        OpCode(0, 0, 0xE, 0xE) => run_00ee(state),
        OpCode(1, _, _, _) => run_1nnn(opcode.get_3n(), state),
        OpCode(2, _, _, _) => run_2nnn(opcode.get_3n(), state),
        OpCode(3, x, _, _) => run_3xnn(x.into(), opcode.get_2n(), state),
        OpCode(4, x, _, _) => run_4xnn(x.into(), opcode.get_2n(), state),
        OpCode(5, x, y, 0) => run_5xy0(x.into(), y.into(), state),
        OpCode(6, x, _, _) => run_6xnn(x.into(), opcode.get_2n(), state),
        OpCode(7, x, _, _) => run_7xnn(x.into(), opcode.get_2n(), state),
        OpCode(9, x, y, 0) => run_9xy0(x.into(), y.into(), state),
        OpCode(0xA, _, _, _) => run_annn(opcode.get_3n(), state),
        OpCode(0xB, x, _, _) => run_bnnn(x.into(), opcode.get_3n(), state, !quirks.has_jumping()),
        OpCode(0xC, x, _, _) => run_cxnn(x.into(), opcode.get_2n(), state),
        OpCode(0xD, x, y, n) => run_dxyn(x.into(), y.into(), n, state, screen),
        OpCode(0xE, x, 9, 0xE) => run_ex9e(x.into(), state, events),
        OpCode(0xE, x, 0xA, 1) => run_exa1(x.into(), state, events),
        OpCode(0xF, x, _, _) => run_fxnn(x.into(), opcode.get_2n(), state, events),
        // Logic Operations
        OpCode(8, x, y, 0) => run_8xy0(x.into(), y.into(), state),
        OpCode(8, x, y, 1) => run_8xy1(x.into(), y.into(), state),
        OpCode(8, x, y, 2) => run_8xy2(x.into(), y.into(), state),
        OpCode(8, x, y, 3) => run_8xy3(x.into(), y.into(), state),
        OpCode(8, x, y, 4) => run_8xy4(x.into(), y.into(), state),
        OpCode(8, x, y, 5) => run_8xy5(x.into(), y.into(), state),
        OpCode(8, x, y, n) => run_8xyn(x.into(), y.into(), n, state, quirks.has_shifting()),
        _ => {}
    }
}

fn run_00e0(screen: &mut Screen) {
    screen.clear()
}

fn run_00ee(state: &mut ChipState) {
    state.pc = state.stack.pop().unwrap()
}

fn run_1nnn(nnn: u16, state: &mut ChipState) {
    state.pc = nnn;
    state.did_jump = true;
}

fn run_2nnn(nnn: u16, state: &mut ChipState) {
    state.stack.push(state.pc);
    state.pc = nnn;
    state.did_jump = true;
}

fn run_3xnn(x: usize, nn: u8, state: &mut ChipState) {
    let vx = state.registers[x];

    if vx == nn {
        state.skip();
    }
}

fn run_4xnn(x: usize, nn: u8, state: &mut ChipState) {
    let vx = state.registers[x];

    if vx != nn {
        state.skip();
    }
}

fn run_5xy0(x: usize, y: usize, state: &mut ChipState) {
    let vx = state.registers[x];
    let vy = state.registers[y];

    if vx == vy {
        state.skip();
    }
}

fn run_6xnn(x: usize, nn: u8, state: &mut ChipState) {
    state.registers[x] = nn;
}

fn run_7xnn(x: usize, nn: u8, state: &mut ChipState) {
    let sum = state.registers[x] as u16 + nn as u16;

    state.registers[x] = sum as u8;
}

fn run_8xy0(x: usize, y: usize, state: &mut ChipState) {
    state.registers[x] = state.registers[y];
    state.registers[15] = 0;
}

fn run_8xy1(x: usize, y: usize, state: &mut ChipState) {
    state.registers[x] = state.registers[x] | state.registers[y];
    state.registers[15] = 0;
}

fn run_8xy2(x: usize, y: usize, state: &mut ChipState) {
    state.registers[x] = state.registers[x] & state.registers[y];
    state.registers[15] = 0;
}

fn run_8xy3(x: usize, y: usize, state: &mut ChipState) {
    state.registers[x] = state.registers[x] ^ state.registers[y];
    state.registers[15] = 0;
}

fn run_8xy4(x: usize, y: usize, state: &mut ChipState) {
    let vx = state.registers[x];
    let vy = state.registers[y];

    let sum = (vx as u16) + (vy as u16);

    state.registers[x] = sum as u8;

    state.registers[15] = if sum > 255 { 1 } else { 0 };
}

fn run_8xy5(x: usize, y: usize, state: &mut ChipState) {
    let vx = state.registers[x];
    let vy = state.registers[y];

    let mut carry = 0u8;

    let sub = if vx < vy {
        ((vx as u16 | 0x100) - (vy as u16)) as u8
    } else {
        carry = 1;
        vx - vy
    };

    state.registers[x] = sub;

    state.registers[15] = carry;
}

fn run_8xy6() {}
fn run_8xy7() {}
fn run_8xye() {}

fn run_8xyn(x: usize, y: usize, n: u8, state: &mut ChipState, change_x: bool) {
    let vx = state.registers[x];
    let vy = state.registers[y];

    let mut carry = 0u8;

    state.registers[x] = match n {
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

fn run_9xy0(x: usize, y: usize, state: &mut ChipState) {
    let vx = state.registers[x];
    let vy = state.registers[y];

    if vx != vy {
        state.skip();
    }
}

fn run_annn(nnn: u16, state: &mut ChipState) {
    state.vi = nnn;
}

fn run_bnnn(x: usize, nnn: u16, state: &mut ChipState, use_x: bool) {
    let x = if use_x { x } else { 0 };

    let address = nnn + state.registers[x as usize] as u16;

    state.pc = address;
    state.did_jump = true;
}

fn run_cxnn(x: usize, nn: u8, state: &mut ChipState) {
    let mut rng = rand::thread_rng();

    let random: u8 = rng.gen();

    state.registers[x] = random & nn;
}

fn run_dxyn(x: usize, y: usize, n: u8, state: &mut ChipState, screen: &mut Screen) {
    let vx = state.registers[x] & (SCREEN_WIDTH - 1);
    let vy = state.registers[y] & (SCREEN_HEIGHT as u8 - 1);

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

fn run_ex9e(x: usize, state: &mut ChipState, events: &mut EventPump) {
    let vx = state.registers[x];
    let key = u8_to_scancode(vx);

    if events
        .keyboard_state()
        .pressed_scancodes()
        .any(|code| code == key)
    {
        state.skip()
    }
}

fn run_exa1(x: usize, state: &mut ChipState, events: &mut EventPump) {
    let vx = state.registers[x];
    let key = u8_to_scancode(vx);

    if events
        .keyboard_state()
        .pressed_scancodes()
        .all(|code| code != key)
    {
        state.skip()
    }
}

fn run_fxnn(x: usize, nn: u8, state: &mut ChipState, events: &mut EventPump) {
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

    let vx = &mut state.registers[x];

    match nn {
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
