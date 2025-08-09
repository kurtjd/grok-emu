mod flags;
mod mcycles;
mod opcodes;
mod opcodes_cb;
//#[cfg(test)]
mod test_helpers;

use grok_bus::BusHandlerZ80;
use opcodes::Opcode;
use std::marker::PhantomData;

type TCycles = u64;

bitflags::bitflags! {
    #[derive(Default, Debug, Copy, Clone, PartialEq)]
    pub struct Flags: u8 {
        // Carry
        const C = (1 << 0);
        // Add/subtract
        const N = (1 << 1);
        // Parity/overflow
        const P = (1 << 2);
        // Undocumented X
        const X = (1 << 3);
        // Half-carry
        const H = (1 << 4);
        // Undocumented Y
        const Y = (1 << 5);
        // Zero
        const Z = (1 << 6);
        // Sign
        const S = (1 << 7);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RegPair {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdxReg {
    IX,
    IY,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Registers {
    /// Program counter
    pub pc: u16,

    /// Stack pointer
    pub sp: u16,

    /// Accumulator
    pub a: u8,
    /// Alternate accumulator
    pub a_: u8,

    /// Status flags
    pub f: Flags,
    /// Alternate status flags
    pub f_: Flags,

    /// GPR B
    pub b: u8,
    /// GPR C
    pub c: u8,
    /// GPR D
    pub d: u8,
    /// GPR E
    pub e: u8,
    /// GPR H
    pub h: u8,
    /// GPR L
    pub l: u8,

    /// Alternate GPR B
    pub b_: u8,
    /// Alternate GPR C
    pub c_: u8,
    /// Alternate GPR D
    pub d_: u8,
    /// Alternate GPR E
    pub e_: u8,
    /// Alternate GPR H
    pub h_: u8,
    /// Alternate GPR L
    pub l_: u8,

    /// Index register X
    pub ix: u16,
    /// Index register Y
    pub iy: u16,

    /// Interrupt vector
    pub i: u8,

    /// Memory refresh
    pub r: u8,

    /// WZ/MEMPTR register
    pub wz: u16,

    /// Instruction register
    pub ir: u8,
    /// Instructon register for current prefix
    pub ir_pre: u8,

    /// Temporary registers
    pub tmp: [u8; 2],

    /// If the last instruction modified flags, contains flags value
    pub q: u8,
}

impl Registers {
    fn get(&self, reg: Reg) -> u8 {
        match reg {
            Reg::A => self.a,
            Reg::F => self.f.bits(),
            Reg::B => self.b,
            Reg::C => self.c,
            Reg::D => self.d,
            Reg::E => self.e,
            Reg::H => self.h,
            Reg::L => self.l,
        }
    }

    fn get_pair(&self, reg_pair: RegPair) -> u16 {
        match reg_pair {
            RegPair::AF => u16::from_be_bytes([self.a, self.f.bits()]),
            RegPair::BC => u16::from_be_bytes([self.b, self.c]),
            RegPair::DE => u16::from_be_bytes([self.d, self.e]),
            RegPair::HL => u16::from_be_bytes([self.h, self.l]),
            RegPair::SP => self.sp,
        }
    }

    fn set(&mut self, reg: Reg, val: u8) {
        match reg {
            Reg::A => self.a = val,
            Reg::F => self.f = Flags::from_bits_retain(val),
            Reg::B => self.b = val,
            Reg::C => self.c = val,
            Reg::D => self.d = val,
            Reg::E => self.e = val,
            Reg::H => self.h = val,
            Reg::L => self.l = val,
        }
    }

    fn set_pair(&mut self, reg_pair: RegPair, val: u16) {
        let bytes = val.to_be_bytes();

        match reg_pair {
            RegPair::AF => (self.a, self.f) = (bytes[0], Flags::from_bits_retain(bytes[1])),
            RegPair::BC => [self.b, self.c] = bytes,
            RegPair::DE => [self.d, self.e] = bytes,
            RegPair::HL => [self.h, self.l] = bytes,
            RegPair::SP => self.sp = val,
        }
    }

    fn exchange(&mut self, reg: Reg) {
        match reg {
            Reg::A => std::mem::swap(&mut self.a, &mut self.a_),
            Reg::F => std::mem::swap(&mut self.f, &mut self.f_),
            Reg::B => std::mem::swap(&mut self.b, &mut self.b_),
            Reg::C => std::mem::swap(&mut self.c, &mut self.c_),
            Reg::D => std::mem::swap(&mut self.d, &mut self.d_),
            Reg::E => std::mem::swap(&mut self.e, &mut self.e_),
            Reg::H => std::mem::swap(&mut self.h, &mut self.h_),
            Reg::L => std::mem::swap(&mut self.l, &mut self.l_),
        }
    }

    fn _get_idx(&self, reg: IdxReg) -> u16 {
        match reg {
            IdxReg::IX => self.ix,
            IdxReg::IY => self.iy,
        }
    }

    fn _set_idx(&mut self, reg: IdxReg, val: u16) {
        match reg {
            IdxReg::IX => self.ix = val,
            IdxReg::IY => self.iy = val,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Interrupts {
    /// True if last executed instruction was an EI
    pub ei: bool,
    /// Interrupt enable flip-flop
    pub iff1: bool,
    /// Backup interrupt enable flip-flop
    pub iff2: bool,
    /// Interrupt mode
    pub im: u8,
    /// Tracks if LD A, i or LD A, r was last executed instruction
    /// If so, P flag reset during interrupt handling
    pub p: bool,
}

pub struct Cpu<B: BusHandlerZ80> {
    reg: Registers,
    int: Interrupts,
    halt: bool,
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
            halt: false,
            tcycle: 0,
            bus: PhantomData,
        }
    }

    /// Advances the CPU one T-cycle
    pub fn tick(&mut self, bus: &mut B) {
        // TODO: Check for Some(int_opcode) and if is Some, execute instead of fetch
        // Also clear int line?
        // TODO: Also handle HALT state
        self.int.ei = false;
        self.int.p = false;
        self.tcycle += 1;
        self.fetch_execute(bus);
    }

    fn fetch_execute(&mut self, bus: &mut B) {
        match self.tcycle {
            1 => self.fetch_t1(bus),
            2 => self.fetch_t2(bus),
            3 => {
                self.reg.ir = self.fetch_t3(bus);
            }
            4 => {
                self.fetch_t4(bus);
                self.execute(Opcode::from(self.reg.ir), bus);
            }
            _ => self.execute(Opcode::from(self.reg.ir), bus),
        }
    }

    fn handle_int(&mut self, _bus: &mut B) {
        self.int.iff1 = false;
        self.int.iff2 = false;

        match self.int.im {
            0 => todo!("Handle IM0 interrupt"),
            1 => todo!("Handle IM1 interrupt"),
            2 => todo!("Handle IM2 interrupt"),
            _ => panic!("Invalid interrupt mode encountered!"),
        }

        // TODO: Store Some(int_opcode) tick() checks next cycle
    }

    fn handle_nmi(&mut self, _bus: &mut B) {
        self.int.iff2 = self.int.iff1;
        self.int.iff1 = false;
        todo!("Handle non-maskable interrupt always");

        // TODO: Store Some(int_opcode) tick() checks next cycle
    }

    fn check_wait(&mut self, bus: &B) {
        if bus.wait() {
            self.tcycle -= 1;
        }
    }
}
