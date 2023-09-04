pub struct Quirks {
    increment_index: bool,
    shifting: bool,
    jumping: bool,
}

impl Quirks {
    pub fn for_chip8() -> Quirks {
        Quirks {
            increment_index: true,
            shifting: true,
            jumping: true,
        }
    }

    pub fn has_increment_index(&self) -> bool {
        self.increment_index
    }

    pub fn has_jumping(&self) -> bool {
        self.jumping
    }

    pub fn has_shifting(&self) -> bool {
        self.shifting
    }
}
