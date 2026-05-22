use grok_6502::bus::{self, Bus, SimpleBus};

const MEM_SIZE: usize = 0x10000;
const ROM_START: usize = 0xC000;
const BANK_RAM_START: usize = 0xD000;
const BANK_RAM_SIZE: usize = 0x1000;
const EXT_RAM_START: usize = 0xE000;
const EXT_RAM_SIZE: usize = 0x2000;

const WRITE_EN_COUNT_MAX: u8 = 1;

const MMIO_START: u16 = 0xC000;
const MMIO_END: u16 = 0xC0FF;

pub(crate) mod address {
    pub const DISK2_START: usize = 0xC600;
    pub const FW_START: usize = 0xD000;
}

mod soft_switch {
    pub const BANK2_RAM_READ_NO_WRITE: usize = 0xC080;
    pub const BANK2_ROM_READ_WRITE: usize = 0xC081;
    pub const BANK2_ROM_READ_NO_WRITE: usize = 0xC082;
    pub const BANK2_RAM_READ_WRITE: usize = 0xC083;
    pub const BANK1_RAM_READ_NO_WRITE: usize = 0xC088;
    pub const BANK1_ROM_READ_WRITE: usize = 0xC089;
    pub const BANK1_ROM_READ_NO_WRITE: usize = 0xC08A;
    pub const BANK1_RAM_READ_WRITE: usize = 0xC08B;

    pub const BANK2_RAM_READ_NO_WRITE_ALT: usize = BANK2_RAM_READ_NO_WRITE + 4;
    pub const BANK2_ROM_READ_WRITE_ALT: usize = BANK2_ROM_READ_WRITE + 4;
    pub const BANK2_ROM_READ_NO_WRITE_ALT: usize = BANK2_ROM_READ_NO_WRITE + 4;
    pub const BANK2_RAM_READ_WRITE_ALT: usize = BANK2_RAM_READ_WRITE + 4;
    pub const BANK1_RAM_READ_NO_WRITE_ALT: usize = BANK1_RAM_READ_NO_WRITE + 4;
    pub const BANK1_ROM_READ_WRITE_ALT: usize = BANK1_ROM_READ_WRITE + 4;
    pub const BANK1_ROM_READ_NO_WRITE_ALT: usize = BANK1_ROM_READ_NO_WRITE + 4;
    pub const BANK1_RAM_READ_WRITE_ALT: usize = BANK1_RAM_READ_WRITE + 4;
}

pub(crate) struct Memory {
    main: [u8; MEM_SIZE],
    bank1_ram: [u8; BANK_RAM_SIZE],
    bank2_ram: [u8; BANK_RAM_SIZE],
    ext_ram: [u8; EXT_RAM_SIZE],
    rom_read: bool,
    ram_write: bool,
    bank2_active: bool,
    write_en_count: u8,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub(crate) fn new() -> Self {
        Memory {
            main: [0; MEM_SIZE],
            bank1_ram: [0; BANK_RAM_SIZE],
            bank2_ram: [0; BANK_RAM_SIZE],
            ext_ram: [0; EXT_RAM_SIZE],
            rom_read: true,
            ram_write: true,
            bank2_active: true,
            write_en_count: WRITE_EN_COUNT_MAX,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.bank2_active = true;
        self.rom_read = true;
        self.ram_write = true;
        self.write_en_count = WRITE_EN_COUNT_MAX;
    }

    pub(crate) fn tick(&mut self, bus: &mut SimpleBus) {
        if (MMIO_START..=MMIO_END).contains(&bus.addr()) {
            return;
        }

        self.handle_soft_sw(bus.addr(), bus.op());
        match bus.op() {
            bus::Op::Read => bus.set_data(self.read(bus.addr())),
            bus::Op::Write => self.write(bus.addr(), bus.data()),
        }
    }

    pub(crate) fn read(&mut self, address: u16) -> u8 {
        match (address as usize) < BANK_RAM_START || self.rom_read {
            true => self.main[address as usize],

            false => match (address as usize) < EXT_RAM_START {
                true => match self.bank2_active {
                    true => self.bank2_ram[address as usize - BANK_RAM_START],
                    false => self.bank1_ram[address as usize - BANK_RAM_START],
                },

                false => self.ext_ram[address as usize - EXT_RAM_START],
            },
        }
    }

    pub(crate) fn data(&self) -> &[u8] {
        &self.main
    }

    pub(crate) fn data_mut(&mut self) -> &mut [u8] {
        &mut self.main
    }

    fn write(&mut self, address: u16, value: u8) {
        if address < ROM_START as u16 {
            self.main[address as usize] = value
        } else if self.ram_write && address >= BANK_RAM_START as u16 {
            if address < EXT_RAM_START as u16 {
                if self.bank2_active {
                    self.bank2_ram[address as usize - BANK_RAM_START] = value;
                } else {
                    self.bank1_ram[address as usize - BANK_RAM_START] = value;
                }
            } else {
                self.ext_ram[address as usize - EXT_RAM_START] = value;
            }
        }
    }

    fn handle_soft_sw(&mut self, address: u16, op: bus::Op) {
        /* Only respond to read requests. But if we receive a write, reset the write enable count
        because technically it requires two READs to become enabled. */
        if op == bus::Op::Write {
            self.write_en_count = WRITE_EN_COUNT_MAX;
            return;
        }

        match address as usize {
            soft_switch::BANK2_RAM_READ_NO_WRITE | soft_switch::BANK2_RAM_READ_NO_WRITE_ALT => {
                self.read_enable(true, false);
            }
            soft_switch::BANK2_ROM_READ_WRITE | soft_switch::BANK2_ROM_READ_WRITE_ALT => {
                self.write_enable(true, true);
            }
            soft_switch::BANK2_ROM_READ_NO_WRITE | soft_switch::BANK2_ROM_READ_NO_WRITE_ALT => {
                self.read_enable(true, true);
            }
            soft_switch::BANK2_RAM_READ_WRITE | soft_switch::BANK2_RAM_READ_WRITE_ALT => {
                self.write_enable(true, false);
            }
            soft_switch::BANK1_RAM_READ_NO_WRITE | soft_switch::BANK1_RAM_READ_NO_WRITE_ALT => {
                self.read_enable(false, false);
            }
            soft_switch::BANK1_ROM_READ_WRITE | soft_switch::BANK1_ROM_READ_WRITE_ALT => {
                self.write_enable(false, true);
            }
            soft_switch::BANK1_ROM_READ_NO_WRITE | soft_switch::BANK1_ROM_READ_NO_WRITE_ALT => {
                self.read_enable(false, true);
            }
            soft_switch::BANK1_RAM_READ_WRITE | soft_switch::BANK1_RAM_READ_WRITE_ALT => {
                self.write_enable(false, false);
            }
            _ => {}
        }
    }

    fn read_enable(&mut self, bank2: bool, rom_read: bool) {
        self.bank2_active = bank2;
        self.rom_read = rom_read;
        self.ram_write = false;
        self.write_en_count = WRITE_EN_COUNT_MAX;
    }

    fn write_enable(&mut self, bank2: bool, rom_read: bool) {
        self.bank2_active = bank2;
        self.rom_read = rom_read;

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
}
