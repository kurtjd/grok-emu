/*
TODO:
-Handle 2nd disc
-Handle writes
-Handle proper reset behavior
*/

mod dsk2woz;
mod woz;

use grok_6502::bus::Bus;
use woz::WozImage;

const MAX_TRACK: u8 = 34;
const MAX_PHASE: usize = 3;

mod soft_switch {
    pub const PHASE0_OFF: u8 = 0;
    pub const PHASE1_OFF: u8 = 0x2;
    pub const PHASE2_OFF: u8 = 0x4;
    pub const PHASE3_OFF: u8 = 0x6;
    pub const DRIVES_OFF: u8 = 0x8;
    pub const SEL_DRIVE1: u8 = 0xA;
    pub const SHIFT_OFF: u8 = 0xC;
    pub const DISK_READ: u8 = 0xE;
    pub const PHASE0_ON: u8 = 0x1;
    pub const PHASE1_ON: u8 = 0x3;
    pub const PHASE2_ON: u8 = 0x5;
    pub const PHASE3_ON: u8 = 0x7;
    pub const DRIVES_ON: u8 = 0x9;
    pub const SEL_DRIVE2: u8 = 0xB;
    pub const SHIFT_ON: u8 = 0xD;
    pub const DISK_WRITE: u8 = 0xF;
}

/// Disk II controller card emulator.
pub struct ControllerCard {
    rom: [u8; 0x100],
    clk: usize,
    data_reg: u8,
    half_track: u8,
    current_phase: usize,
    phases: [bool; MAX_PHASE + 1],
    bit_pntr: usize,
    reading_byte: bool,
    drives_on: bool,
    current_drive: u8,
    write_mode: bool,
    write_sense: bool,
    disk_image: Option<WozImage>,
    motor_off_delay: usize,
}

impl ControllerCard {
    pub fn new(rom: [u8; 0x100], clk: usize) -> Self {
        ControllerCard {
            rom,
            clk,
            data_reg: 0,
            half_track: 0,
            current_phase: 0,
            phases: [false; MAX_PHASE + 1],
            bit_pntr: 0,
            reading_byte: false,
            drives_on: false,
            current_drive: 1,
            write_mode: false,
            write_sense: false,
            disk_image: None,
            motor_off_delay: 0,
        }
    }

    pub fn insert_woz(&mut self, data: &[u8]) {
        self.load_image(WozImage::new(data).unwrap());
    }

    pub fn insert_dsk(&mut self, data: &[u8]) {
        self.load_image(WozImage::new_dsk(data).unwrap());
    }

    pub fn insert_po(&mut self, data: &[u8]) {
        self.load_image(WozImage::new_po(data).unwrap());
    }

    pub fn load_image(&mut self, woz: WozImage) {
        self.disk_image = Some(woz);
    }

    fn handle_motor_off_delay(&mut self) {
        // There is actually a one second delay before drives actually turn off
        if self.motor_off_delay > 0 {
            self.motor_off_delay -= 1;

            if self.motor_off_delay == 0 {
                self.drives_on = false;
            }
        }
    }

    fn step_motor(&mut self, to: usize) {
        let from = self.current_phase;
        let ascending = (to > from && to - from < MAX_PHASE) || (to == 0 && from == MAX_PHASE);
        let descending = (to < from) || (to == MAX_PHASE && from == 0);

        if ascending && self.half_track < MAX_TRACK * 2 {
            self.half_track += 1;
        } else if descending && self.half_track > 0 {
            self.half_track -= 1;
        }

        self.current_phase = to;
    }

    fn phase_on(&mut self, phase: usize) {
        self.phases[phase] = true;

        // If the current phase is OFF, move here
        if !self.phases[self.current_phase] {
            self.step_motor(phase);
        }
    }

    fn phase_off(&mut self, phase: usize) {
        self.phases[phase] = false;

        /* If we just turned off the current phase, but there's a neighboring ON phase,
        then move there */
        if self.current_phase == phase {
            let right_phase = match self.current_phase < MAX_PHASE {
                true => self.current_phase + 1,
                false => 0,
            };
            let left_phase = match self.current_phase > 0 {
                true => self.current_phase - 1,
                false => MAX_PHASE,
            };

            if self.phases[right_phase] {
                self.step_motor(right_phase);
            } else if self.phases[left_phase] {
                self.step_motor(left_phase);
            }
        }
    }

    fn get_next_bit(&mut self) -> u8 {
        // Figure out what track we are on
        let track_idx = (self.half_track / 2) as usize;
        let track = &(self.disk_image.as_ref().unwrap().tracks[track_idx]);
        let track_data = &track.data;

        // Then figure out which byte in the track we are on
        let byte_idx = self.bit_pntr / 8;
        let byte = track_data[byte_idx];

        // And finally figure out what bit in that byte we are on
        let bit_on = self.bit_pntr % 8;
        let bit = (byte >> (7 - bit_on)) & 1;

        // Wrap around to simulate disk spinning in circle
        self.bit_pntr += 1;
        self.bit_pntr %= track.bit_count as usize;

        bit
    }

    fn load_bit(&mut self) {
        let mut bit = self.get_next_bit();

        if !self.reading_byte {
            /* If we receive a 0, we are in the middle of a 10-bit self-sync byte so keep reading
            until at the beginning of a valid disk byte */
            while bit == 0 {
                bit = self.get_next_bit();
            }
            self.reading_byte = true;
        }

        self.data_reg <<= 1;
        self.data_reg |= bit;
    }

    fn read_bit(&mut self, bus: &mut dyn Bus) {
        if !self.drives_on {
            return;
        }

        if !self.write_mode {
            // If in write-protect sense mode, return whether or not disk is write protected
            if self.write_sense {
                self.data_reg = match self.disk_image.as_ref().unwrap().write_protected {
                    true => 1 << 7,
                    false => 0,
                };
            } else {
                self.load_bit();
            }
        }

        // Put the contents of the register on the data bus
        bus.set_data(self.data_reg);

        // If the high bit is set, we've finished reading in a disk byte so clear register
        if self.data_reg & (1 << 7) != 0 {
            self.data_reg = 0;
            self.reading_byte = false;
        }
    }
}

impl crate::peripheral::Peripheral for ControllerCard {
    fn tick(&mut self, _bus: &mut dyn Bus, _pins: &mut crate::peripheral::Pins) {
        self.handle_motor_off_delay();
    }

    fn device_select(&mut self, bus: &mut dyn Bus, _pins: &mut crate::peripheral::Pins) {
        if self.disk_image.is_none() {
            return;
        }

        let addr = (bus.addr() & 0xF) as u8;
        match addr {
            // Off
            soft_switch::PHASE0_OFF => {
                self.phase_off(0);
                self.read_bit(bus);
            }
            soft_switch::PHASE1_OFF => {
                self.phase_off(1);
                self.read_bit(bus);
            }
            soft_switch::PHASE2_OFF => {
                self.phase_off(2);
                self.read_bit(bus);
            }
            soft_switch::PHASE3_OFF => {
                self.phase_off(3);
                self.read_bit(bus);
            }
            soft_switch::DRIVES_OFF => {
                self.motor_off_delay = self.clk;
                self.read_bit(bus);
            }
            soft_switch::SEL_DRIVE1 => {
                self.current_drive = 1;
                self.read_bit(bus);
            }
            soft_switch::SHIFT_OFF => {
                self.write_sense = false;
                if !self.write_mode {
                    self.read_bit(bus);
                } else {
                    // TODO: Actually write data to disk image
                    // Copy data reg to disk byte pointer
                    // I assume CPU waits for sequencer to shift out bits?
                    // So can just do it in one go?
                }
            }
            soft_switch::DISK_READ => {
                self.write_mode = false;
                self.read_bit(bus);
            }

            // On
            soft_switch::PHASE0_ON => {
                self.phase_on(0);
            }
            soft_switch::PHASE1_ON => {
                self.phase_on(1);
            }
            soft_switch::PHASE2_ON => {
                self.phase_on(2);
            }
            soft_switch::PHASE3_ON => {
                self.phase_on(3);
            }
            soft_switch::DRIVES_ON => {
                self.drives_on = true;
                self.motor_off_delay = 0;
            }
            soft_switch::SEL_DRIVE2 => {
                self.current_drive = 2;
            }
            soft_switch::SHIFT_ON => {
                self.write_sense = true;
                if self.write_mode {
                    self.data_reg = bus.data();
                } else {
                    self.data_reg = 0; // Apprently reading this addr clears data register
                }
            }
            soft_switch::DISK_WRITE => {
                self.write_mode = true;
            }
            _ => {}
        }
    }

    fn io_select(&mut self, bus: &mut dyn Bus, _pins: &mut crate::peripheral::Pins) {
        let addr = (bus.addr() & 0xFF) as usize;
        bus.set_data(self.rom[addr]);
    }

    fn io_strobe(&mut self, _bus: &mut dyn Bus, _pins: &mut super::Pins) {
        // Intentionally do nothing
    }
}
