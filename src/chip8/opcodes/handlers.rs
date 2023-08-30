use crate::chip8::opcodes::parser;
use crate::chip8::quirks::Quirks;
use crate::chip8::state::ChipState;
use crate::screen::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::Rng;

pub fn run(opcode: u16, state: &mut ChipState, quirks: Quirks, screen: &mut Screen) {
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
        0xB => run_bnnn(opcode, state, quirks.has_jumping()),
        0xC => run_cxnn(opcode, state),
        0xD => run_dxyn(opcode, state, screen),
        0xE => (),
        0xF => (),
        _ => {}
    }
}

fn run_00en(opcode: u16, state: &mut ChipState, screen: &mut Screen) {
    match parser::get_n(opcode) {
        0x0 => screen.clear(),
        0xE => state.pc = state.stack.pop().unwrap(),
        _ => {}
    }
}

fn run_1nnn(opcode: u16, state: &mut ChipState) {
    state.pc = parser::get_3n(opcode);
    state.did_jump = true;
}

fn run_2nnn(opcode: u16, state: &mut ChipState) {
    state.stack.push(state.pc);
    state.pc = parser::get_3n(opcode);
    state.did_jump = true;
}

fn run_3xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;

    let vx = state.memory[x];

    if vx == parser::get_2n(opcode) {
        state.skip();
    }
}

fn run_4xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;

    let vx = state.memory[x];

    if vx != parser::get_2n(opcode) {
        state.skip();
    }
}

fn run_5xy0(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    let y = parser::get_y(opcode) as usize;
    let vy = state.registers[y];

    if x == y {
        state.skip();
    }
}

fn run_6xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;

    state.registers[x] = parser::get_2n(opcode);
}

fn run_7xnn(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;

    state.registers[x] += parser::get_2n(opcode);
}

fn run_8xyn(opcode: u16, state: &mut ChipState, change_x: bool) {
    let x = parser::get_x(opcode) as usize;
    let y = parser::get_y(opcode) as usize;
    let op = parser::get_n(opcode);

    let vx = state.registers[x];
    let vy = state.registers[y];

    state.registers[15] = 0;

    state.registers[x] = match op {
        0x0 => vy,
        0x1 => vx | vy,
        0x2 => vx & vy,
        0x3 => vx ^ vy,
        0x4 => {
            let sum = (vx as u16) + (vy as u16);
            if sum > 255 {
                state.registers[15] = 1; // Set carry flag
            }
            sum as u8
        }
        0x5 => {
            state.registers[15] = 1;
            if vx < vy {
                state.registers[15] = 0;
                ((vx as u16 | 0x100) - (vy as u16)) as u8
            } else {
                ((vx as u16) - (vy as u16)) as u8
            }
        }
        0x6 => {
            state.registers[15] = vx & 1;
            if change_x {
                vy >> 1
            } else {
                vx >> 1
            }
        }
        0x7 => {
            state.registers[15] = 1;
            if vy < vx {
                state.registers[15] = 0;
                ((vy as u16 | 0x100) - (vx as u16)) as u8
            } else {
                ((vy as u16) - (vx as u16)) as u8
            }
        }
        0xE => {
            state.registers[15] = (vx & 0x80) >> 7;
            if change_x {
                vy >> 1
            } else {
                vx >> 1
            }
        }
    }
}

fn run_9xy0(opcode: u16, state: &mut ChipState) {
    let x = parser::get_x(opcode) as usize;
    let vx = state.registers[x];

    let y = parser::get_y(opcode) as usize;
    let vy = state.registers[y];

    if x != y {
        state.skip();
    }
}

fn run_annn(opcode: u16, state: &mut ChipState) {
    state.vi = parser::get_3n(opcode);
}

fn run_bnnn(opcode: u16, state: &mut ChipState, use_x: bool) {
    let x = if use_x { parser::get_x(opcode) } else { 0 };

    let address = parser::get_3n(opcode) + state.registers[x as usize] as u16;

    state.pc = address;
    state.did_jump = true;
}

fn run_cxnn(opcode: u16, state: &mut ChipState) {
    let mut rng = rand::thread_rng();

    let x = parser::get_x(opcode) as usize;

    let random: u8 = rng.gen();

    state.registers[x] = random & parser::get_2n(opcode);
}

fn run_dxyn(opcode: u16, state: &mut ChipState, screen: &mut Screen) {
    state.should_draw = true;

    let x = parser::get_x(opcode) as usize;
    let y = parser::get_y(opcode) as usize;
    let n = parser::get_n(opcode) as usize;

    state.registers[15] = 0;

    for index in 0..n {
        let wrapped_position = (y + index) & (SCREEN_HEIGHT - 1);
        let line = &mut screen.screen_memory[wrapped_position];

        let address = state.vi + index as u16;

        let sprite = state.memory[address as usize] as u64;

        let offset_sprite = sprite << (SCREEN_WIDTH - 8) >> x;

        let new_line = *line ^ offset_sprite;

        for bit in 0..SCREEN_WIDTH {
            let bit_before = *line & (0x8000000000000000 >> bit);
            let bit_after = new_line & (0x8000000000000000 >> bit);

            if (bit_before != bit_after) && (bit_before >> (SCREEN_WIDTH - 1 - bit)) == 1 {
                state.registers[0xF] = 1;
            }
        }

        *line = new_line;
    }
}
