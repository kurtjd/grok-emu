mod disk;
pub mod graphics;
mod input;
mod memory;
pub mod sound;

use disk::controller::Controller;
use graphics::{Graphics, Video};
use grok_6502::Cpu;
use grok_6502::bus::{Bus, SimpleBus};
use input::Input;
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
    input: Input,
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
        let input = Input::new();
        let disk = Controller::new(settings::DISK_SLOT);

        Apple2 {
            bus,
            memory,
            graphics,
            sound,
            input,
            disk,
            cpu,
        }
    }

    pub fn reset(&mut self) {
        self.memory.reset();
        self.disk.reset();
        self.cpu.reset(&mut self.bus);
    }

    pub fn init(&mut self) {
        self.load_fw_rom();
        self.load_disk_rom();
        self.reset();
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
            self.input.tick(&mut self.bus);
            self.bus.tick();
            self.cpu.tick(&mut self.bus);
        }

        self.sound.feed_samples();
        self.disk.handle_motor_off_delay();
    }

    pub fn input(&mut self, char: u8, shift: bool, ctrl: bool) {
        self.input.set(char, shift, ctrl);
    }

    pub fn input_arrow(&mut self, right: bool) {
        self.input.set_arrow(right);
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
