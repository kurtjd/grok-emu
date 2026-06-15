//! Apple II Language Card

use grok_6502::bus::{self, Bus};

const WRITE_EN_COUNT_MAX: u8 = 1;

const BANK_RAM_SIZE: usize = 0x1000;
const BANK_RAM_START: usize = 0xD000;
const BANK_RAM_END: usize = BANK_RAM_START + BANK_RAM_SIZE;

const EXT_RAM_SIZE: usize = 0x2000;
const EXT_RAM_START: usize = 0xE000;
const EXT_RAM_END: usize = EXT_RAM_START + EXT_RAM_SIZE;

mod soft_switch {
    pub const BANK2_RAM_READ_NO_WRITE: u8 = 0x0;
    pub const BANK2_ROM_READ_WRITE: u8 = 0x1;
    pub const BANK2_ROM_READ_NO_WRITE: u8 = 0x2;
    pub const BANK2_RAM_READ_WRITE: u8 = 0x3;
    pub const BANK1_RAM_READ_NO_WRITE: u8 = 0x8;
    pub const BANK1_ROM_READ_WRITE: u8 = 0x9;
    pub const BANK1_ROM_READ_NO_WRITE: u8 = 0xA;
    pub const BANK1_RAM_READ_WRITE: u8 = 0xB;

    pub const BANK2_RAM_READ_NO_WRITE_ALT: u8 = BANK2_RAM_READ_NO_WRITE + 4;
    pub const BANK2_ROM_READ_WRITE_ALT: u8 = BANK2_ROM_READ_WRITE + 4;
    pub const BANK2_ROM_READ_NO_WRITE_ALT: u8 = BANK2_ROM_READ_NO_WRITE + 4;
    pub const BANK2_RAM_READ_WRITE_ALT: u8 = BANK2_RAM_READ_WRITE + 4;
    pub const BANK1_RAM_READ_NO_WRITE_ALT: u8 = BANK1_RAM_READ_NO_WRITE + 4;
    pub const BANK1_ROM_READ_WRITE_ALT: u8 = BANK1_ROM_READ_WRITE + 4;
    pub const BANK1_ROM_READ_NO_WRITE_ALT: u8 = BANK1_ROM_READ_NO_WRITE + 4;
    pub const BANK1_RAM_READ_WRITE_ALT: u8 = BANK1_RAM_READ_WRITE + 4;
}

/// Language card emulator.
pub struct LanguageCard {
    bank1_ram: [u8; BANK_RAM_SIZE],
    bank2_ram: [u8; BANK_RAM_SIZE],
    ext_ram: [u8; EXT_RAM_SIZE],
    ram_read: bool,
    ram_write: bool,
    bank2_active: bool,
    write_en_count: u8,
}

impl Default for LanguageCard {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageCard {
    pub fn new() -> Self {
        LanguageCard {
            bank1_ram: [0; BANK_RAM_SIZE],
            bank2_ram: [0; BANK_RAM_SIZE],
            ext_ram: [0; EXT_RAM_SIZE],
            ram_read: false,
            ram_write: false,
            bank2_active: true,
            write_en_count: WRITE_EN_COUNT_MAX,
        }
    }

    fn read_enable(&mut self, bank2: bool, ram_read: bool) {
        self.bank2_active = bank2;
        self.ram_read = ram_read;
        self.ram_write = false;
        self.write_en_count = WRITE_EN_COUNT_MAX;
    }

    fn write_enable(&mut self, bank2: bool, ram_read: bool) {
        self.bank2_active = bank2;
        self.ram_read = ram_read;

        // It takes two consecutive accesses to a write enable switch to actually enable RAM write
        if !self.ram_write {
            if self.write_en_count == 0 {
                self.ram_write = true;
                self.write_en_count = WRITE_EN_COUNT_MAX;
            } else {
                self.write_en_count -= 1;
            }
        }
    }

    fn read(&mut self, bus: &mut dyn Bus) {
        let addr = bus.addr() as usize;

        match addr {
            BANK_RAM_START..BANK_RAM_END => {
                let bank_addr = addr - BANK_RAM_START;
                if self.bank2_active {
                    bus.set_data(self.bank2_ram[bank_addr])
                } else {
                    bus.set_data(self.bank1_ram[bank_addr])
                }
            }
            EXT_RAM_START..EXT_RAM_END => bus.set_data(self.ext_ram[addr - EXT_RAM_START]),
            _ => (),
        }
    }

    fn write(&mut self, bus: &mut dyn Bus) {
        let addr = bus.addr() as usize;
        let data = bus.data();

        match addr {
            BANK_RAM_START..BANK_RAM_END => {
                let bank_addr = addr - BANK_RAM_START;
                if self.bank2_active {
                    self.bank2_ram[bank_addr] = data;
                } else {
                    self.bank1_ram[bank_addr] = data;
                }
            }
            EXT_RAM_START..EXT_RAM_END => self.ext_ram[addr - EXT_RAM_START] = data,
            _ => (),
        }
    }
}

impl super::Peripheral for LanguageCard {
    fn tick(&mut self, bus: &mut dyn Bus, pins: &mut super::Pins) {
        match bus.op() {
            bus::Op::Read if self.ram_read => {
                pins.set_inh(true);
                self.read(bus);
            }
            bus::Op::Write if self.ram_write => {
                pins.set_inh(true);
                self.write(bus);
            }
            _ => pins.set_inh(false),
        }
    }

    fn io_select(&mut self, _bus: &mut dyn Bus, _pins: &mut super::Pins) {
        // Do nothing, language card has no ROM
    }

    fn device_select(&mut self, bus: &mut dyn Bus, _pins: &mut super::Pins) {
        // If we receive a write, reset the write enable count because technically
        // it requires two consecutive READs to become enabled
        if bus.op() == bus::Op::Write {
            self.write_en_count = WRITE_EN_COUNT_MAX;
        }

        match (bus.addr() & 0xF) as u8 {
            soft_switch::BANK2_RAM_READ_NO_WRITE | soft_switch::BANK2_RAM_READ_NO_WRITE_ALT => {
                self.read_enable(true, true);
            }
            soft_switch::BANK2_ROM_READ_WRITE | soft_switch::BANK2_ROM_READ_WRITE_ALT => {
                self.write_enable(true, false);
            }
            soft_switch::BANK2_ROM_READ_NO_WRITE | soft_switch::BANK2_ROM_READ_NO_WRITE_ALT => {
                self.read_enable(true, false);
            }
            soft_switch::BANK2_RAM_READ_WRITE | soft_switch::BANK2_RAM_READ_WRITE_ALT => {
                self.write_enable(true, true);
            }
            soft_switch::BANK1_RAM_READ_NO_WRITE | soft_switch::BANK1_RAM_READ_NO_WRITE_ALT => {
                self.read_enable(false, true);
            }
            soft_switch::BANK1_ROM_READ_WRITE | soft_switch::BANK1_ROM_READ_WRITE_ALT => {
                self.write_enable(false, false);
            }
            soft_switch::BANK1_ROM_READ_NO_WRITE | soft_switch::BANK1_ROM_READ_NO_WRITE_ALT => {
                self.read_enable(false, false);
            }
            soft_switch::BANK1_RAM_READ_WRITE | soft_switch::BANK1_RAM_READ_WRITE_ALT => {
                self.write_enable(false, true);
            }
            _ => {}
        }
    }

    fn io_strobe(&mut self, _bus: &mut dyn Bus, _pins: &mut super::Pins) {
        // Intentionally do nothing
    }
}
