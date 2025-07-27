mod mcycles;
mod opcode;
//#[cfg(test)]
mod test_helpers;

use crate::opcode::Opcode;
use grok_bus::BusHandlerZ80;
use std::marker::PhantomData;

type TCycles = u64;

// General-purpose registers
#[derive(Default, Copy, Clone, Debug)]
pub struct Gpr {
    // Accumulator
    pub a: u8,
    // Status flags
    pub f: u8,
    // B
    pub b: u8,
    // C
    pub c: u8,
    // D
    pub d: u8,
    // E
    pub e: u8,
    // H
    pub h: u8,
    // L
    pub l: u8,
}

// Special-purpose registers
#[derive(Default, Copy, Clone, Debug)]
pub struct Spr {
    // Program counter
    pub pc: u16,
    // Stack pointer
    pub sp: u16,

    // IX
    pub ix: u16,
    // IY
    pub iy: u16,

    // Interrupt vector
    pub i: u8,
    // Memory refresh
    pub r: u8,
}

// Temporary working registers
#[derive(Default, Copy, Clone, Debug)]
pub struct Wpr {
    // Work register W
    pub w: u8,
    // Work register Z
    pub z: u8,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Registers {
    pub spr: Spr,
    pub gpr: Gpr,
    pub gpr_alt: Gpr,
    pub wpr: Wpr,
    // Instruction register
    pub ir: u8,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Interrupts {
    pub ei: bool,
    pub iff1: bool,
    pub iff2: bool,
    pub im: u8,
}

pub struct Cpu<B: BusHandlerZ80> {
    reg: Registers,
    int: Interrupts,
    tcycle: TCycles,
    bus: PhantomData<B>,
}

impl<B: BusHandlerZ80> Default for Cpu<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: BusHandlerZ80> Cpu<B> {
    /// Creates a new CPU
    pub fn new() -> Self {
        Self {
            reg: Registers::default(),
            int: Interrupts::default(),
            tcycle: 0,
            bus: PhantomData,
        }
    }

    /// Advances the CPU one T-cycle
    pub fn tick(&mut self, bus: &mut B) {
        if self.tcycle == 0 {
            self.int.ei = false;
        }

        self.tcycle += 1;

        match self.tcycle {
            1 => self.fetch_t1(bus),
            2 => self.fetch_t2(bus),
            3 => self.fetch_t3(bus),
            4 => {
                self.fetch_t4(bus);
                self.execute(Opcode::from(self.reg.ir), bus);
            }
            _ => self.execute(Opcode::from(self.reg.ir), bus),
        }
    }

    fn check_wait(&mut self, bus: &B) {
        if bus.wait() {
            self.tcycle -= 1;
        }
    }

    fn reg_pair(&self, reg1: u8, reg2: u8) -> u16 {
        u16::from_be_bytes([reg1, reg2])
    }
}
