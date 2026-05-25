pub mod io;
mod memory;
pub mod peripheral;

pub use io::{Audio, Video};

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
    pub const CPU_CLK_SPEED: u32 = 1024000;
    pub const DISP_WIDTH: u32 = 280;
    pub const DISP_HEIGHT: u32 = 192;
    pub const DISP_SCALE: u32 = 3;
    pub const PIXEL_SIZE: u32 = 3;
    pub const SAMPLE_RATE: u32 = 44100;
}

use grok_6502::Cpu;
use grok_6502::bus::{Bus, SimpleBus};
use io::Io;
use io::graphics::{CHAR_ROM_SIZE, Graphics};
use io::keyboard::Keyboard;
use io::speaker::Speaker;
use memory::{ROM_SIZE, Ram, Rom};
use peripheral::{Peripheral, Peripherals};

pub struct Apple2<'a, V: Video, A: Audio> {
    bus: SimpleBus,
    cpu: Cpu,
    rom: Rom,
    ram: Ram,
    io: Io<V, A>,
    peripherals: Peripherals<'a>,
}

impl<'a, V: Video, A: Audio> Apple2<'a, V, A> {
    pub fn new(fw_rom: [u8; ROM_SIZE], char_rom: [u8; CHAR_ROM_SIZE], video: V, audio: A) -> Self {
        let bus = SimpleBus::new();
        let cpu = Cpu::new();

        let ram = Ram::new();
        let rom = Rom::new(fw_rom);

        let graphics = Graphics::new(char_rom, video);
        let speaker = Speaker::new(audio);
        let keyboard = Keyboard::new();
        let io = Io {
            keyboard,
            graphics,
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

    pub fn run_frame(&mut self, frame_rate: u32) {
        let cycles_per_frame = settings::CPU_CLK_SPEED / frame_rate;

        // Note: Not very accurate since we only snapshot the graphics memory at the end of frame,
        // but wanna make this update per cpu cycle so we can emulate those racing the beam tricks
        self.io.graphics.draw(frame_rate, &self.ram[..]);

        // Tick the various components for this frame
        for _ in 0..cycles_per_frame {
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

        // We've collected samples during the frame, so feed them to the audio output
        self.io.speaker.feed_samples();
    }

    pub fn input(&mut self, char: u8, shift: bool, ctrl: bool) {
        self.io.keyboard.input(char, shift, ctrl);
    }

    pub fn input_arrow(&mut self, right: bool) {
        self.io.keyboard.input_arrow(right);
    }

    pub fn insert_peripheral(&mut self, peripheral: &'a mut dyn Peripheral, slotno: usize) {
        assert!(
            slotno < self.peripherals.slots.len(),
            "Slot number must be between 0 and 7"
        );
        self.peripherals.slots[slotno] = Some(peripheral);
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
