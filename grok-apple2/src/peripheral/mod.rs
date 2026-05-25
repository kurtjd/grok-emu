pub mod disk;
pub mod language;

use crate::mem_map;
use grok_6502::bus::Bus;

const NUM_SLOTS: usize = 8;

/// Shared pins between all peripherals and the motherboard.
pub struct Pins {
    inh: bool,
}

impl Pins {
    /// Returns whewther the INH (inhibit) pin is currently active.
    ///
    /// If active, motherboard ROM should not drive the data bus.
    pub fn inh(&self) -> bool {
        self.inh
    }

    /// Sets the active state of the INH (inhibit) pin.
    ///
    /// If set active, motherboard ROM should not drive the data bus.
    pub fn set_inh(&mut self, inh: bool) {
        self.inh = inh;
    }

    pub(crate) fn new() -> Self {
        Self { inh: false }
    }
}

pub(crate) struct Peripherals<'a> {
    pub(crate) pins: Pins,
    // Chose to own references instead of boxing it up to keep this no_std compatible
    pub(crate) slots: [Option<&'a mut dyn Peripheral>; NUM_SLOTS],
}

impl Default for Peripherals<'_> {
    fn default() -> Self {
        Peripherals {
            pins: Pins::new(),
            slots: [const { None }; NUM_SLOTS],
        }
    }
}

impl Peripherals<'_> {
    pub(crate) fn tick(&mut self, bus: &mut dyn Bus) {
        for peripheral in self.slots.iter_mut().flatten() {
            peripheral.tick(bus, &mut self.pins);
        }
    }

    pub(crate) fn device_select(&mut self, bus: &mut dyn Bus) {
        let slot = (bus.addr() - mem_map::DEVICE_SELECT) / 0x10;

        if let Some(peripheral) = &mut self.slots[slot as usize] {
            peripheral.device_select(bus, &mut self.pins);
        }
    }

    pub(crate) fn io_select(&mut self, bus: &mut dyn Bus) {
        // Slot 0 does not have ROM space
        let slot = ((bus.addr() - mem_map::IO_SELECT) / 0x100) + 1;

        if let Some(peripheral) = &mut self.slots[slot as usize] {
            peripheral.io_select(bus, &mut self.pins);
        }
    }

    pub(crate) fn decode(&mut self, bus: &mut dyn Bus) {
        match bus.addr() {
            mem_map::DEVICE_SELECT..mem_map::IO_SELECT => self.device_select(bus),
            mem_map::IO_SELECT..mem_map::IO_STROBE => self.io_select(bus),
            mem_map::IO_STROBE.. => (),
            _ => unreachable!(),
        }
    }
}

/// Interface for various Apple II peripheral cards that aren't built directly into the motherboard.
///
/// This can be used to emulate a wide range of peripherals.
pub trait Peripheral {
    /// Called every CPU cycle, regardless of whether the peripheral is selected or not.
    ///
    /// Useful for updating internal state on a consistent basis.
    fn tick(&mut self, bus: &mut dyn Bus, pins: &mut Pins);

    /// Called when the peripheral's ROM space ($Cnxx) is selected.
    ///
    /// In this case, the peripheral should drive the data bus with the appropriate ROM data.
    fn io_select(&mut self, bus: &mut dyn Bus, pins: &mut Pins);

    /// Called when the peripheral's I/O space ($C0nx) is selected.
    fn device_select(&mut self, bus: &mut dyn Bus, pins: &mut Pins);
}
