//! MOS 6502 bus trait.
//!
//! The idea is that all hardware devices (including the CPU) only communicate over one shared
//! bus (typically with the CPU driving the address lines, the memory module driving the data
//! lines, and other hardware snooping the lines).
//!
//! This is technailly more than just the bus, as it includes other pins/signals
//! from the 6502 cpu, but `Bus` still seems like best overall name.

/// The operation currently being performed on the bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Op {
    /// Write operation.
    #[default]
    Write,
    /// Read operation.
    Read,
}

/// MOS 6502 bus/pins/signals trait.
///
/// *Note*: Some signals differ in polarity (active high vs active low) on hardware,
/// but this interface abstracts that through just a single `active` bool.
/// So essentially from this interface all signals are active high for simplicity.
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

    /// Set the SYNC pin level.
    fn set_sync(&mut self, active: bool);

    /// Get the current level of the SYNC pin.
    fn sync(&self) -> bool;

    /// Get the current edge (if any) of the SYNC pin.
    fn sync_edge(&self) -> Option<bool>;

    /// Set the RES pin level.
    fn set_res(&mut self, active: bool);

    /// Get the current level of the RES pin.
    fn res(&self) -> bool;

    /// Get the current edge (if any) of the RES pin.
    fn res_edge(&self) -> Option<bool>;

    /// Set the IRQ pin level.
    fn set_irq(&mut self, active: bool);

    /// Get the current level of the IRQ pin.
    fn irq(&self) -> bool;

    /// Get the current edge (if any) of the IRQ pin.
    fn irq_edge(&self) -> Option<bool>;

    /// Set the NMI pin level.
    fn set_nmi(&mut self, active: bool);

    /// Get the current level of the NMI pin.
    fn nmi(&self) -> bool;

    /// Get the current edge (if any) of the NMI pin.
    fn nmi_edge(&self) -> Option<bool>;

    /// Set the S.O. pin level.
    fn set_so(&mut self, active: bool);

    /// Get the current level of the S.O. pin.
    fn so(&self) -> bool;

    /// Get the current edge (if any) of the S.O. pin.
    fn so_edge(&self) -> Option<bool>;

    /// Set the RDY pin level.
    fn set_rdy(&mut self, active: bool);

    /// Get the current level of the RDY pin.
    fn rdy(&self) -> bool;

    /// Get the current edge (if any) of the RDY pin.
    fn rdy_edge(&self) -> Option<bool>;

    /// Advance the bus one cycle.
    ///
    /// Typically used to update edge state of pins.
    fn tick(&mut self);

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

#[derive(Debug, Clone, Copy)]
struct Pin {
    prev_level: bool,
    level: bool,
    edge: Option<bool>,
}

impl Pin {
    // Create a new `Pin` with the given initial level.
    fn new(level: bool) -> Self {
        Self {
            prev_level: level,
            level,
            edge: None,
        }
    }

    // Tick the pin (basically just update its edge state)
    fn tick(&mut self) {
        self.edge = if self.level != self.prev_level {
            Some(self.level)
        } else {
            None
        };
        self.prev_level = self.level;
    }
}

/// A simple implementation of the `Bus` trait that just stores the current state of the bus in fields.
#[derive(Debug, Clone, Copy)]
pub struct SimpleBus {
    addr: u16,
    data: u8,
    op: Op,
    sync: Pin,
    res: Pin,
    irq: Pin,
    nmi: Pin,
    so: Pin,
    rdy: Pin,
}

impl Default for SimpleBus {
    fn default() -> Self {
        Self {
            addr: 0,
            data: 0,
            op: Op::Write,
            sync: Pin::new(false),
            res: Pin::new(false),
            irq: Pin::new(false),
            nmi: Pin::new(false),
            so: Pin::new(false),
            rdy: Pin::new(true),
        }
    }
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

    fn set_sync(&mut self, active: bool) {
        self.sync.level = active;
    }

    fn sync(&self) -> bool {
        self.sync.level
    }

    fn sync_edge(&self) -> Option<bool> {
        self.sync.edge
    }

    fn set_res(&mut self, active: bool) {
        self.res.level = active;
    }

    fn res(&self) -> bool {
        self.res.level
    }

    fn res_edge(&self) -> Option<bool> {
        self.res.edge
    }

    fn set_irq(&mut self, active: bool) {
        self.irq.level = active;
    }

    fn irq(&self) -> bool {
        self.irq.level
    }

    fn irq_edge(&self) -> Option<bool> {
        self.irq.edge
    }

    fn set_nmi(&mut self, active: bool) {
        self.nmi.level = active;
    }

    fn nmi(&self) -> bool {
        self.nmi.level
    }

    fn nmi_edge(&self) -> Option<bool> {
        self.nmi.edge
    }

    fn set_so(&mut self, active: bool) {
        self.so.level = active;
    }

    fn so(&self) -> bool {
        self.so.level
    }

    fn so_edge(&self) -> Option<bool> {
        self.so.edge
    }

    fn set_rdy(&mut self, active: bool) {
        self.rdy.level = active;
    }

    fn rdy(&self) -> bool {
        self.rdy.level
    }

    fn rdy_edge(&self) -> Option<bool> {
        self.rdy.edge
    }

    fn tick(&mut self) {
        self.sync.tick();
        self.res.tick();
        self.irq.tick();
        self.nmi.tick();
        self.so.tick();
        self.rdy.tick();
    }
}
