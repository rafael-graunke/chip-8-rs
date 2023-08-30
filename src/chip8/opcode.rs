pub struct OpCode {
    code: u16,
}

impl OpCode {
    pub fn init() -> OpCode {
        OpCode { code: 0u16 }
    }

    pub fn update(&mut self, pc: usize, memory: &[u8; 4096]) {
        let first_byte = (memory[pc] as u16) << 8;
        let second_byte = memory[pc + 1] as u16;

        self.code = first_byte | second_byte;
    }

    pub fn mask(&self, mask: u16) -> u16 {
        self.code & mask
    }

    pub fn get_x(&self) -> u8 {
        ((self.code & 0x0F00) >> 8) as u8
    }

    pub fn get_y(&self) -> u8 {
        ((self.code & 0x00F0) >> 4) as u8
    }

    pub fn get_n(&self) -> u8 {
        (self.code & 0x000F) as u8
    }

    pub fn get_2n(&self) -> u8 {
        (self.code & 0x00FF) as u8
    }

    pub fn get_3n(&self) -> u16 {
        self.code & 0x0FFF
    }
}
