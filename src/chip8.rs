use rand::Rng;
use std::fs::File;
use std::io::prelude::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: usize = 32;
const PIXEL_SCALE: usize = 20;
const MEM_OFFSET: u16 = 0x200;
const FONT_OFFSET: u16 = 0x50;

struct Display {
    screen_memory: [u64; SCREEN_HEIGHT],
    display: Canvas<Window>,
}

impl Display {
    fn init_canvas(sdl: &Sdl) -> Canvas<Window> {
        let video_subsystem = sdl.video().unwrap();

        let window = video_subsystem
            .window(
                "rust-sdl2 demo: Video",
                SCREEN_WIDTH as u32 * PIXEL_SCALE as u32,
                SCREEN_HEIGHT as u32 * PIXEL_SCALE as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(Color::RGB(38, 17, 13));
        canvas.clear();

        canvas
    }

    pub fn new(sdl: &Sdl) -> Display {
        Display {
            screen_memory: [0u64; SCREEN_HEIGHT],
            display: Display::init_canvas(sdl),
        }
    }

    pub fn clear(&mut self) {
        self.screen_memory = [0u64; SCREEN_HEIGHT];
    }
}

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
        let mut pixel = Rect::new(0, 0, PIXEL_SCALE as u32, PIXEL_SCALE as u32);

        let width = SCREEN_WIDTH as u64;

        for (row_index, row) in self.display.screen_memory.iter().enumerate() {
            for column in 0..width {
                if 1u64 << (width - 1 - column) & row != 0 {
                    pixel.x = column as i32 * PIXEL_SCALE as i32;
                    pixel.y = row_index as i32 * PIXEL_SCALE as i32;
                    self.display.display.set_draw_color(Color::RGB(155, 66, 49));
                    self.display.display.fill_rect(pixel).unwrap();
                }
            }
        }

        self.display.display.present();
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
        self.update_opcode();
        self.run_opcode();

        if !self.did_jump {
            self.pc += 2;
        };

        self.did_jump = false;
    }

    /* opcode functions (might change(?)) */
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
