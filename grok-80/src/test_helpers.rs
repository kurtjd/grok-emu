use crate::{Cpu, Interrupts, Registers, TCycles};
use grok_bus::BusHandlerZ80;

impl<B: BusHandlerZ80> Cpu<B> {
    pub fn reg(&self) -> Registers {
        self.reg
    }

    pub fn set_reg(&mut self, reg: Registers) {
        self.reg = reg;
    }

    pub fn int(&self) -> Interrupts {
        self.int
    }

    pub fn set_int(&mut self, int: Interrupts) {
        self.int = int;
    }

    pub fn tcycle(&self) -> TCycles {
        self.tcycle
    }
}
