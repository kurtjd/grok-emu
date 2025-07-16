//! Emulation of the bus used by Space Invaders
use crate::input_reg::InputReg;
use crate::shift_reg::ShiftReg;
use crate::sound_reg::SoundReg;
use grok_bus::BusHandler;

pub(crate) struct Bus {
    ram: [u8; 0x100],
    shift_reg: ShiftReg,
    pub(crate) input_reg: InputReg,
    pub(crate) sound_reg: SoundReg,
}

impl Bus {
    pub(crate) fn new() -> Self {
        Self {
            ram: [0x00; 0x100],
            shift_reg: ShiftReg::new(),
            input_reg: InputReg::new(),
            sound_reg: SoundReg::new(),
        }
    }
}

impl BusHandler for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }

    fn port_read(&mut self, port: u8) -> u8 {
        match port {
            1 => self.input_reg.read_reg1(),
            2 => self.input_reg.read_reg2(),
            3 => self.shift_reg.read(),
            _ => {
                println!("Unexpected IN port: {port}");
                0
            }
        }
    }

    fn port_write(&mut self, port: u8, val: u8) {
        match port {
            2 => self.shift_reg.write_amnt(val),
            3 => self.sound_reg.set_reg1(val),
            4 => self.shift_reg.write(val),
            5 => self.sound_reg.set_reg2(val),
            6 => {
                // Called by Space Invaders to reset watchdog timer
                // Treat as NOP since we have no need to emulate a watchdog timer
            }
            _ => {
                println!("Unexpected OUT port: {port}");
            }
        }
    }
}
