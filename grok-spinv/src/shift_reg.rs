//! Emulation of the shift register chip used by Space Invaders
pub(crate) struct ShiftReg {
    reg: u16,
    amnt: u8,
}

impl ShiftReg {
    pub(crate) fn new() -> Self {
        Self { reg: 0, amnt: 0 }
    }

    pub(crate) fn read(&self) -> u8 {
        (self.reg >> (8 - self.amnt)) as u8
    }

    pub(crate) fn write(&mut self, val: u8) {
        self.reg >>= 8;
        self.reg |= (val as u16) << 8;
    }

    pub(crate) fn write_amnt(&mut self, val: u8) {
        self.amnt = val & 7;
    }
}
