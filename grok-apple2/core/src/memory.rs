use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;
use grok_6502::bus::{self, Bus};

const RAM_SIZE: usize = 48 * 1024;
pub(crate) const ROM_SIZE: usize = 12 * 1024;

pub(crate) struct Ram {
    data: [u8; RAM_SIZE],
}

impl Ram {
    pub(crate) fn new() -> Self {
        Ram {
            data: [0; RAM_SIZE],
        }
    }

    pub(crate) fn decode(&mut self, bus: &mut dyn Bus) {
        match bus.op() {
            bus::Op::Read => bus.set_data(self[bus.addr() as usize]),
            bus::Op::Write => self[bus.addr() as usize] = bus.data(),
        }
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for Ram {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for Ram {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index]
    }
}

pub(crate) struct Rom {
    data: [u8; ROM_SIZE],
}

impl Rom {
    pub(crate) fn new(data: [u8; ROM_SIZE]) -> Self {
        Rom { data }
    }

    pub(crate) fn decode(&mut self, bus: &mut dyn Bus, periph_pins: &mut crate::peripheral::Pins) {
        // Don't want ROM driving the bus if a peripheral has set the INH pin active this cycle
        if !periph_pins.inh() {
            let rom_addr = bus.addr() - crate::mem_map::ROM;
            bus.set_data(self[rom_addr as usize]);
        }
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for Rom {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for Rom {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index]
    }
}
