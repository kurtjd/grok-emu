pub mod io;
mod memory;
pub mod peripheral;

mod mem_map {
    pub const RAM: u16 = 0x0000;

    // Various IO components built directly into the motherboard
    pub const BUILTIN_IO: u16 = 0xC000;
    pub const KEYBOARD_EN: u16 = 0xC000;
    pub const KEYBOARD_CLR: u16 = 0xC010;
    pub const CASSETTE_TOGGLE: u16 = 0xC020;
    pub const SPEAKER: u16 = 0xC030;
    pub const UTIL_STROBE: u16 = 0xC040;
    pub const SCREEN_MODE: u16 = 0xC050;
    pub const ANNUNCIATOR: u16 = 0xC058;
    pub const CASSETTE_IN: u16 = 0xC060;
    pub const PUSHBTN_IN: u16 = 0xC061;
    pub const CONTROLLER_IN: u16 = 0xC064;
    pub const CASSETTE_IN_ALT: u16 = 0xC068;
    pub const PUSHBTN_IN_ALT: u16 = 0xC069;
    pub const CONTROLLER_IN_ALT: u16 = 0xC06C;
    pub const TIMER_TRIGGER: u16 = 0xC070;

    // Address space for peripheral card IO
    pub const PERIPHERAL_IO: u16 = 0xC080;
    pub const DEVICE_SELECT: u16 = 0xC080;
    pub const IO_SELECT: u16 = 0xC100;
    pub const IO_STROBE: u16 = 0xC800;

    pub const ROM: u16 = 0xD000;
}

pub mod settings {
    use crate::io::video;

    pub const CPU_CLK_SPEED: u32 = 1024000;
    pub const DISP_WIDTH: u32 = 280;
    pub const DISP_HEIGHT: u32 = 192;
    pub const DISP_SCALE: u32 = 3;
    pub const SAMPLE_RATE: u32 = 44100;
    pub const CYCLES_PER_FRAME: u32 = video::VSCAN_MAX as u32 * video::HSCAN_MAX as u32;
}

use grok_6502::Cpu;
use grok_6502::bus::{Bus, SimpleBus};
use io::Io;
use io::keyboard::Keyboard;
use io::speaker::Speaker;
use io::video::{self, CHAR_ROM_SIZE, Video};
use memory::{ROM_SIZE, Ram, Rom};
use peripheral::{Peripheral, PeripheralHandle, Peripherals};

pub struct Frame<'a> {
    pub video: &'a [u32],
    pub audio: &'a [bool],
}

pub struct Apple2<'a> {
    bus: SimpleBus,
    cpu: Cpu,
    rom: Rom,
    ram: Ram,
    io: Io,
    peripherals: Peripherals<'a>,
}

impl<'a> Apple2<'a> {
    pub fn new(fw_rom: [u8; ROM_SIZE], char_rom: [u8; CHAR_ROM_SIZE]) -> Self {
        let bus = SimpleBus::new();
        let cpu = Cpu::new();

        let ram = Ram::new();
        let rom = Rom::new(fw_rom);

        let video = Video::new(char_rom);
        let speaker = Speaker::new();
        let keyboard = Keyboard::new();
        let io = Io {
            keyboard,
            video,
            speaker,
            game: Default::default(),
        };

        let peripherals = Peripherals::default();

        Apple2 {
            bus,
            cpu,
            rom,
            ram,
            io,
            peripherals,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
    }

    pub fn init(&mut self) {
        self.reset();
    }

    pub fn run_frame(&mut self) -> Frame<'_> {
        self.io.speaker.begin_frame();

        // Tick the various components for this frame
        for vscan in 0..video::VSCAN_MAX {
            for hscan in 0..video::HSCAN_MAX {
                // Maybe a sort of hackish way to emulate cycle stealing for video
                // Basically in reality a memory access takes only one clock phase,
                // so video can access memory on the first phase and then cpu on the next.
                // But, we sort of have memory accesses take 2 phases in code because
                // we need a chance for the memory module to see the bus activity between
                // driving the address bus and reading the data bus.
                //
                // So: Tick the video twice, then tick cpu twice and handle all other updates there.
                // It is safe to call `decode` here because video will only put RAM addresses on the bus.
                self.io.video.tick(vscan, hscan, &mut self.bus);
                self.decode();
                self.io.video.tick(vscan, hscan, &mut self.bus);

                // Tick the CPU one clock phase so it can announce address on the bus
                self.cpu.tick(&mut self.bus);

                // Then give peripherals a chance to react first in case they need to inhibit ROM
                self.peripherals.tick(&mut self.bus);

                // Update the speaker state
                self.io.speaker.tick();

                // Update the bus state
                self.bus.tick();

                // Then decode the address and dispatch to appropriate component
                self.decode();

                // Then finally tick the CPU one more clock phase to react to the data bus
                self.cpu.tick(&mut self.bus);
            }
        }

        // Return the frame video and audio samples
        let video = self.io.video.render();
        let audio = self.io.speaker.samples();
        Frame { video, audio }
    }

    pub fn input(&mut self, char: u8, shift: bool, ctrl: bool) {
        self.io.keyboard.input(char, shift, ctrl);
    }

    pub fn input_arrow(&mut self, right: bool) {
        self.io.keyboard.input_arrow(right);
    }

    pub fn insert_peripheral(&mut self, peripheral: PeripheralHandle<'a>, slotno: usize) {
        self.peripherals.slots[slotno] = Some(peripheral);
    }

    pub fn remove_peripheral(&mut self, slotno: usize) {
        self.peripherals.slots[slotno] = None;
    }

    pub fn peripheral_mut<T: Peripheral + 'static>(&mut self, slotno: usize) -> Option<&mut T> {
        self.peripherals.slots[slotno]
            .as_deref_mut()?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    pub fn peripheral_ref<T: Peripheral + 'static>(&self, slotno: usize) -> Option<&T> {
        self.peripherals.slots[slotno]
            .as_deref()?
            .as_any()
            .downcast_ref::<T>()
    }

    fn decode(&mut self) {
        match self.bus.addr() {
            mem_map::RAM..mem_map::BUILTIN_IO => self.ram.decode(&mut self.bus),
            mem_map::BUILTIN_IO..mem_map::PERIPHERAL_IO => self.io.decode(&mut self.bus),
            mem_map::PERIPHERAL_IO..mem_map::ROM => self.peripherals.decode(&mut self.bus),
            mem_map::ROM.. => self.rom.decode(&mut self.bus, &mut self.peripherals.pins),
        }
    }
}
