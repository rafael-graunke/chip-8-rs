pub fn mask(opcode: u16, mask: u16) -> u16 {
    opcode & mask
}

pub fn get_digit(opcode: u16) -> u8 {
    (mask(opcode, 0xF000) >> 12) as u8
}

pub fn get_x(opcode: u16) -> u8 {
    (mask(opcode, 0x0F00) >> 8) as u8
}

pub fn get_y(opcode: u16) -> u8 {
    (mask(opcode, 0x00F0) >> 4) as u8
}

pub fn get_n(opcode: u16) -> u8 {
    mask(opcode, 0x000F) as u8
}

pub fn get_2n(opcode: u16) -> u8 {
    mask(opcode, 0x00FF) as u8
}

pub fn get_3n(opcode: u16) -> u16 {
    mask(opcode, 0x0FFF)
}
