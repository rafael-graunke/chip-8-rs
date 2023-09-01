pub struct OpCode(pub u8, pub u8, pub u8, pub u8);

impl OpCode {
    pub fn from(code: u16) -> OpCode {
        OpCode(get_digit(code), get_x(code), get_y(code), get_n(code))
    }

    pub fn get_2n(&self) -> u8 {
        (self.2 << 4) | self.3
    }

    pub fn get_3n(&self) -> u16 {
        ((self.1 as u16) << 8) | ((self.2 as u16) << 4) | self.3 as u16
    }
}

fn mask(opcode: u16, mask: u16) -> u16 {
    opcode & mask
}

fn get_digit(opcode: u16) -> u8 {
    (mask(opcode, 0xF000) >> 12) as u8
}

fn get_x(opcode: u16) -> u8 {
    (mask(opcode, 0x0F00) >> 8) as u8
}

fn get_y(opcode: u16) -> u8 {
    (mask(opcode, 0x00F0) >> 4) as u8
}

fn get_n(opcode: u16) -> u8 {
    mask(opcode, 0x000F) as u8
}
