//! MOS 6502 bus trait.
//!
//! The idea is that all hardware devices (including the CPU) only communicate over one shared
//! bus (typically with the CPU driving the address lines, the memory module driving the data
//! lines, and other hardware snooping the lines).
//!
//! This is technailly more than just the bus, as it includes other pins/signals
//! from the 6502 cpu, but `Bus` still seems like best overall name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Op {
    #[default]
    Write,
    Read,
}

/// MOS 6502 bus/pins/signals trait.
pub trait Bus {
    /// Set the address lines to the given value.
    fn set_addr(&mut self, addr: u16);
    /// Get the current value of the address lines.
    fn addr(&self) -> u16;
    /// Set the data lines to the given value.
    fn set_data(&mut self, data: u8);
    /// Get the current value of the data lines.
    fn data(&self) -> u8;
    /// Set the current bus operation (read or write).
    fn set_op(&mut self, op: Op);
    /// Get the current bus operation (read or write).
    fn op(&self) -> Op;
    /// Set the SYNC signal (active high when instruction is in T0 phase).
    fn set_sync(&mut self, sync: bool);
    /// Get the current state of the SYNC signal (active high when instruction is in T0 phase).
    fn sync(&self) -> bool;
    /// Set the RES signal (active low) causing a reset of the CPU.
    fn set_res(&mut self, res: bool);
    /// Get the current state of the RES signal (active low).
    fn res(&self) -> bool;

    /// Helper method to start a read operation on the bus.
    fn start_read(&mut self, addr: u16) {
        self.set_addr(addr);
        self.set_op(Op::Read);
    }

    /// Helper method to start a write operation on the bus.
    fn start_write(&mut self, addr: u16, data: u8) {
        self.set_addr(addr);
        self.set_data(data);
        self.set_op(Op::Write);
    }
}

/// A simple implementation of the `Bus` trait that just stores the current state of the bus in fields.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleBus {
    addr: u16,
    data: u8,
    op: Op,
    sync: bool,
    res: bool,
}

impl SimpleBus {
    /// Create a new `SimpleBus` with default (0) values.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Bus for SimpleBus {
    fn set_addr(&mut self, addr: u16) {
        self.addr = addr;
    }

    fn addr(&self) -> u16 {
        self.addr
    }

    fn set_data(&mut self, data: u8) {
        self.data = data;
    }

    fn data(&self) -> u8 {
        self.data
    }

    fn set_op(&mut self, op: Op) {
        self.op = op;
    }

    fn op(&self) -> Op {
        self.op
    }

    fn set_sync(&mut self, sync: bool) {
        self.sync = sync;
    }

    fn sync(&self) -> bool {
        self.sync
    }

    fn set_res(&mut self, res: bool) {
        self.res = res;
    }

    fn res(&self) -> bool {
        self.res
    }
}
