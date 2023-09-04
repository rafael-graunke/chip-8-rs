#[derive(Debug)]
pub struct ChipState {
    pub memory: [u8; 4096],
    pub stack: Vec<u16>,
    pub registers: [u8; 16],
    pub vi: u16,
    pub pc: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub should_draw: bool,
    pub running: bool,
    pub did_jump: bool,
    pub should_wait: bool,
}

impl ChipState {
    pub fn init() -> ChipState {
        // Initialize memory
        let mut memory = [0u8; 4096];

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
            let address = index + 0x50 as usize;
            memory[address] = *byte;
        }

        ChipState {
            memory,
            stack: vec![],
            registers: [0u8; 16],
            vi: 0u16,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            should_draw: false,
            running: true,
            did_jump: false,
            should_wait: false,
        }
    }

    pub fn skip(&mut self) {
        self.pc += 2;
    }
}
