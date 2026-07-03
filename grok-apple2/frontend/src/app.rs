use crate::audio::{self, Machine};
use crate::gui::{self, UiAction};
use crate::input;
use crate::serial::StdSerialPort;
use crate::video::Display;
use eframe::egui;
use grok_apple2_core::peripheral::serial::SuperSerial;
use grok_apple2_core::peripheral::{Peripheral, disk, language};
use grok_apple2_core::{Apple2, settings};
use std::time::{Duration, Instant};

// The correct ROM files must be placed at the paths below
// They are not included since they are technically still under copyright
// So make sure you only use ROMs that you have the legal right to use (lol)
const FW_ROM: [u8; 0x3000] = *include_bytes!("../roms/apple2_plus.rom");
const CHAR_ROM: [u8; 0x800] = *include_bytes!("../roms/char_set.rom");
const DISK2_ROM: [u8; 0x100] = *include_bytes!("../roms/disk2.rom");
const SSC_ROM: [u8; 0x800] = *include_bytes!("../roms/ssc.rom");

const FRAME_RATE: u64 = 60;
const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / FRAME_RATE);

pub struct App {
    apple2: Machine,
    display: Display,
    next_frame: Instant,
    // Held to keep the audio stream alive for the lifetime of the app.
    _audio_stream: cpal::Stream,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, disk_path: Option<String>) -> Self {
        let (audio, audio_stream) = audio::init_audio();

        // Peripherals are borrowed by the emulator for its whole lifetime, so we
        // leak them to obtain 'static references and avoid a self-referential App.
        let language_card: &'static mut _ = Box::leak(Box::new(language::LanguageCard::new()));

        // Setup for ADTPro
        let sw1 = 0b1111001;
        let sw2 = 0b0011011;
        let serial_card: &'static mut _ = Box::leak(Box::new(SuperSerial::new(
            StdSerialPort::new(),
            SSC_ROM,
            sw1,
            sw2,
        )));

        let disk_card: &'static mut _ = Box::leak(Box::new(disk::ControllerCard::new(
            DISK2_ROM,
            settings::CPU_CLK_SPEED as usize,
        )));

        // Insert disk if one was passed on the command line.
        if let Some(path) = disk_path {
            insert_disk(disk_card, &path);
        }

        let mut apple2 = Apple2::new(FW_ROM, CHAR_ROM, audio);
        apple2.insert_peripheral(language_card as &mut dyn Peripheral, 0);
        apple2.insert_peripheral(serial_card as &mut dyn Peripheral, 2);
        apple2.insert_peripheral(disk_card as &mut dyn Peripheral, 6);
        apple2.init();

        App {
            apple2,
            display: Display::new(&cc.egui_ctx),
            next_frame: Instant::now(),
            _audio_stream: audio_stream,
        }
    }
}

/// Read a disk image from `path` and load it into the controller, dispatching on
/// the file extension. Panics on an unreadable file or unknown format.
fn insert_disk(card: &mut disk::ControllerCard, path: &str) {
    let buffer = std::fs::read(path).expect("failed to read disk image");
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "woz" => card.insert_woz(&buffer),
        "dsk" => card.insert_dsk(&buffer),
        "po" => card.insert_po(&buffer),
        _ => panic!("Unsupported disk format: .{ext}"),
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        input::handle_input(&mut self.apple2, &ctx);

        // Advance the emulation at a fixed 60 Hz regardless of how often eframe
        // repaints (input events can trigger extra repaints).
        if Instant::now() >= self.next_frame {
            let frame = self.apple2.run_frame();
            self.display.update(frame);

            self.next_frame += FRAME_TIME;
            // If we've fallen behind, resync rather than trying to catch up forever.
            let now = Instant::now();
            if self.next_frame < now {
                self.next_frame = now + FRAME_TIME;
            }
        }

        egui::Panel::top("toolbar").show(ui, |ui| {
            if let Some(action) = gui::menu_bar(ui) {
                match action {
                    UiAction::Reset => self.apple2.reset(),
                    UiAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                }
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            self.display.draw(ui);
        });

        // Wake us up in time for the next emulated frame.
        let wait = self.next_frame.saturating_duration_since(Instant::now());
        ctx.request_repaint_after(wait);
    }
}
