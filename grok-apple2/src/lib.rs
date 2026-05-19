mod disk;
pub mod graphics;
mod input;
mod memory;
pub mod sound;

use disk::controller::Controller;
use graphics::{Graphics, Video};
use grok_6502::Cpu;
use grok_6502::bus::{Bus, SimpleBus};
use memory::Memory;
use sound::{Audio, Sound};

// The correct ROM files must be added to this repo at the paths below
// They are not included since they are technically still under copyright
// So make sure you only use ROMs that you have the legal right to use (lol)
const FW_ROM: &[u8] = include_bytes!("../roms/firmware/apple2_plus.rom");
const DISK_ROM: &[u8] = include_bytes!("../roms/firmware/disk2.rom");
const CHAR_ROM: &[u8] = include_bytes!("../roms/firmware/char_set.rom");

mod settings {
    pub const CPU_CLK_SPEED: u32 = 1024000;
    pub const DISK_SLOT: usize = 0x60;
}

pub struct Apple2<V: Video, A: Audio> {
    bus: SimpleBus,
    memory: Memory,
    graphics: Graphics<V>,
    sound: Sound<A>,
    disk: Controller,
    cpu: Cpu,
}

impl<V: Video, A: Audio> Apple2<V, A> {
    pub fn new(video: V, audio: A) -> Self {
        let bus = SimpleBus::new();
        let memory = Memory::new();
        let cpu = Cpu::new();
        let graphics = Graphics::new(video);
        let sound = Sound::new(audio);
        let disk = Controller::new(settings::DISK_SLOT);

        Apple2 {
            bus,
            memory,
            graphics,
            sound,
            disk,
            cpu,
        }
    }

    pub fn reset(&mut self) {
        self.memory.reset();
        self.disk.reset();
        self.cpu.reset();
    }

    pub fn init(&mut self) {
        self.load_fw_rom();
        self.load_disk_rom();
        self.cpu.reset();
    }

    pub fn input(&mut self, char: u8, shift: bool, ctrl: bool) {
        // Convert lowercase to uppercase
        let mut ascii = char;
        if ascii.is_ascii_lowercase() {
            ascii -= 32;
        }

        // Get the proper ASCII character if shift held
        if shift {
            ascii = input::get_shift_ascii(ascii);
        }

        // Do nothing if not a valid Apple 2 key
        if !input::is_valid_key(ascii) {
            return;
        }

        // Modify the value (if necessary) when CTRL is held
        if ctrl {
            ascii = input::get_ctrl_ascii(ascii);
        }

        // The Apple 2 has the high bit set for ASCII characters
        self.memory.data_mut()[input::DATA_ADDR] = ascii | (1 << 7);
    }

    pub fn input_arrow(&mut self, right: bool) {
        let ascii = if right {
            input::KEY_RIGHT
        } else {
            input::KEY_LEFT
        };
        self.memory.data_mut()[input::DATA_ADDR] = ascii;
    }

    pub fn run_frame(&mut self, frame_rate: u32) {
        let cycles_per_frame = settings::CPU_CLK_SPEED / frame_rate;

        // Note: Not very accurate since we only snapshot the graphics memory at the end of frame,
        // but wanna make this update per cpu cycle so we can emulate those racing the beam tricks
        self.graphics.update(frame_rate, self.memory.data());

        // Tick the CPU for this frame
        for _ in 0..cycles_per_frame {
            self.cpu.tick(&mut self.bus);
            self.memory.tick(&mut self.bus);
            self.graphics.tick(&mut self.bus);
            self.sound.tick(&self.bus);
            self.disk.tick(&mut self.bus);
            input::tick(&mut self.bus, self.memory.data_mut());
            self.bus.tick();
            self.cpu.tick(&mut self.bus);
        }

        self.sound.feed_samples();
        self.disk.handle_motor_off_delay();
    }

    pub fn insert_woz(&mut self, data: &[u8]) {
        self.disk
            .load_image(disk::woz::WozImage::new(data).unwrap());
    }

    pub fn insert_dsk(&mut self, data: &[u8]) {
        self.disk
            .load_image(disk::woz::WozImage::new_dsk(data).unwrap());
    }

    pub fn insert_po(&mut self, data: &[u8]) {
        self.disk
            .load_image(disk::woz::WozImage::new_po(data).unwrap());
    }

    fn load_fw_rom(&mut self) {
        self.memory.data_mut()[memory::address::FW_START..memory::address::FW_START + FW_ROM.len()]
            .copy_from_slice(FW_ROM);
    }

    fn load_disk_rom(&mut self) {
        self.memory.data_mut()
            [memory::address::DISK2_START..memory::address::DISK2_START + DISK_ROM.len()]
            .copy_from_slice(DISK_ROM);
    }
}
