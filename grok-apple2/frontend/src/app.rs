use crate::audio::{self, CpalAudio, Machine};
use crate::gui::{self, CardKind, SerialConfig, SerialPortSource, UiAction};
use crate::input;
use crate::serial::StdSerialPort;
use crate::video::Display;
use eframe::egui;
use grok_apple2_core::peripheral::serial::SuperSerial;
use grok_apple2_core::peripheral::{Peripheral, disk, language};
use grok_apple2_core::{Apple2, settings};
use std::sync::atomic::{AtomicBool, Ordering};
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
    // Frontend-side mirror of which card occupies each slot, used to label the
    // Cards menu and highlight the active card. The core has no "what kind?"
    // query, so the frontend tracks it here as the source of truth.
    slots: [Option<CardKind>; 8],
    // Name of the disk image loaded into each slot's Drive 1 (if any), shown in
    // the menu. Cleared when the slot's card is removed or replaced.
    disk_names: [Option<String>; 8],
    // Raw disk image bytes (and format) currently loaded into each slot, kept so
    // a power cycle can re-insert the same disk into a freshly built card. On
    // real hardware the floppy stays in the drive across a power cycle.
    disk_images: [Option<StoredDisk>; 8],
    // User-editable Super Serial Card settings (port source + DIP switches),
    // applied whenever an SSC is built.
    serial: SerialConfig,
    // A handle to the audio ring buffer shared with the live output stream, used
    // to build a fresh sink for the emulator when it's rebuilt on a power cycle.
    audio_ring: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<f32>>>,
    // Mutes the audio output stream when set. Lives in the (never-rebuilt) output
    // stream, so muting persists across power cycles.
    audio_muted: std::sync::Arc<AtomicBool>,
    // When true, emulation is frozen: `run_frame` is skipped so the display holds
    // the last frame and the audio ring drains to silence.
    paused: bool,
    // Held to keep the audio stream alive for the lifetime of the app.
    _audio_stream: cpal::Stream,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (audio, audio_stream, audio_muted) = audio::init_audio();
        // Keep a handle to the ring so a power cycle can build a new sink for the
        // still-running output stream rather than restarting the audio device.
        let audio_ring = audio.ring();

        // Default layout: Language Card in slot 0, Disk II controller in slot 6.
        let mut slots = [None; 8];
        slots[0] = Some(CardKind::LanguageCard);
        slots[6] = Some(CardKind::DiskController);

        // Disks are inserted at runtime through the Slots menu.
        let disk_names: [Option<String>; 8] = std::array::from_fn(|_| None);
        let disk_images: [Option<StoredDisk>; 8] = std::array::from_fn(|_| None);

        let serial = SerialConfig::default();
        let apple2 = build_machine(audio, &slots, &serial, &disk_images);

        App {
            apple2,
            display: Display::new(&cc.egui_ctx),
            next_frame: Instant::now(),
            slots,
            disk_names,
            disk_images,
            serial,
            audio_ring,
            audio_muted,
            paused: false,
            _audio_stream: audio_stream,
        }
    }

    /// Simulate turning the Apple II off and back on: tear down the running
    /// machine and build a brand-new one, giving cleared RAM, reset soft
    /// switches, reset peripheral state, and a fresh CPU. Inserted disks are
    /// re-inserted (the floppy stays in the drive across a power cycle); the
    /// audio stream is reattached via the shared ring buffer.
    fn power_cycle(&mut self) {
        let audio = CpalAudio::new(self.audio_ring.clone());
        self.apple2 = build_machine(audio, &self.slots, &self.serial, &self.disk_images);
        self.next_frame = Instant::now();
    }

    /// Freeze or resume emulation. When resuming, the frame clock is reset to now
    /// so the machine picks up from the current frame instead of stampeding
    /// through a backlog of "missed" frames accrued while paused.
    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if !paused {
            self.next_frame = Instant::now();
        }
    }
}

/// The format of a disk image, tracked so a cached image can be re-inserted into
/// a freshly built controller without re-reading or re-sniffing the file.
#[derive(Clone, Copy)]
enum DiskFormat {
    Woz,
    Dsk,
    Po,
}

/// A disk image the frontend has loaded, retained so a power cycle can put the
/// same disk back into the rebuilt controller.
struct StoredDisk {
    format: DiskFormat,
    data: Vec<u8>,
}

/// Build a fresh emulator from the current slot/serial/disk configuration. Used
/// both at startup and on every power cycle.
fn build_machine(
    audio: CpalAudio,
    slots: &[Option<CardKind>; 8],
    serial: &SerialConfig,
    disk_images: &[Option<StoredDisk>; 8],
) -> Machine {
    let mut apple2 = Apple2::new(FW_ROM, CHAR_ROM, audio);
    for (slot, kind) in slots.iter().enumerate() {
        let Some(kind) = *kind else { continue };
        apple2.insert_peripheral(build_card(kind, serial), slot);
        // Re-insert any cached disk image into the freshly built controller.
        if let Some(image) = &disk_images[slot]
            && let Some(card) = apple2.peripheral_mut::<disk::ControllerCard>(slot)
        {
            insert_disk_bytes(card, &image.data, image.format);
        }
    }
    apple2.init();
    apple2
}

/// Construct a fresh peripheral card of `kind`, running any real-world setup the
/// card needs (e.g. the Super Serial Card spins up a PTY).
fn build_card(kind: CardKind, serial: &SerialConfig) -> Box<dyn Peripheral> {
    match kind {
        CardKind::LanguageCard => Box::new(language::LanguageCard::new()),
        CardKind::Serial => build_serial(serial),
        CardKind::DiskController => Box::new(disk::ControllerCard::new(
            DISK2_ROM,
            settings::CPU_CLK_SPEED as usize,
        )),
    }
}

/// Build a Super Serial Card from the user's current configuration, opening the
/// requested host-side port and applying the DIP switch settings.
fn build_serial(serial: &SerialConfig) -> Box<dyn Peripheral> {
    let port = match &serial.source {
        SerialPortSource::Pty => StdSerialPort::pty(),
        SerialPortSource::Path(path) => StdSerialPort::open(path),
    };
    Box::new(SuperSerial::new(port, SSC_ROM, serial.sw1, serial.sw2))
}

/// Read a disk image from `path`, sniffing the format from the file extension.
/// Logs and returns `None` on an unreadable file or unknown format rather than
/// crashing the emulator. The returned image is cached by the caller so it can
/// survive a power cycle.
fn read_disk(path: &std::path::Path) -> Option<StoredDisk> {
    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read disk image {}: {e}", path.display());
            return None;
        }
    };
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some("woz") => DiskFormat::Woz,
        Some("dsk") => DiskFormat::Dsk,
        Some("po") => DiskFormat::Po,
        other => {
            eprintln!("Unsupported disk format: {other:?}");
            return None;
        }
    };
    Some(StoredDisk { format, data })
}

/// Insert already-read disk bytes into the controller, dispatching on the cached
/// format.
fn insert_disk_bytes(card: &mut disk::ControllerCard, data: &[u8], format: DiskFormat) {
    match format {
        DiskFormat::Woz => card.insert_woz(data),
        DiskFormat::Dsk => card.insert_dsk(data),
        DiskFormat::Po => card.insert_po(data),
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Host-side keyboard shortcuts (power cycle, pause, mute) can't be
        // handled inside `handle_input` since they touch the whole app, not just
        // the machine, so they bubble up as requests here. Emulated keys are
        // swallowed while paused.
        let requests = input::handle_input(&mut self.apple2, &ctx, self.paused);
        if requests.power_cycle {
            self.power_cycle();
        }
        if requests.toggle_pause {
            self.set_paused(!self.paused);
        }
        if requests.toggle_mute {
            self.audio_muted.fetch_xor(true, Ordering::Relaxed);
        }

        // Advance the emulation at a fixed 60 Hz regardless of how often eframe
        // repaints (input events can trigger extra repaints). Skipped while
        // paused so the machine freezes on the current frame.
        if !self.paused && Instant::now() >= self.next_frame {
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
            if let Some(action) = gui::menu_bar(
                ui,
                &self.slots,
                &self.disk_names,
                &mut self.serial,
                self.paused,
                self.audio_muted.load(Ordering::Relaxed),
            ) {
                match action {
                    UiAction::Reset => self.apple2.reset(),
                    UiAction::PowerCycle => self.power_cycle(),
                    UiAction::TogglePause => self.set_paused(!self.paused),
                    UiAction::ToggleMute => {
                        self.audio_muted.fetch_xor(true, Ordering::Relaxed);
                    }
                    UiAction::InsertCard { slot, kind } => {
                        self.apple2
                            .insert_peripheral(build_card(kind, &self.serial), slot);
                        self.slots[slot] = Some(kind);
                        self.disk_names[slot] = None;
                        self.disk_images[slot] = None;
                    }
                    UiAction::ReconfigureSerial { slot } => {
                        // Config was already updated in place by the menu; tear
                        // down and rebuild the card so the change takes effect.
                        self.apple2.remove_peripheral(slot);
                        self.apple2
                            .insert_peripheral(build_serial(&self.serial), slot);
                    }
                    UiAction::RemoveCard { slot } => {
                        self.apple2.remove_peripheral(slot);
                        self.slots[slot] = None;
                        self.disk_names[slot] = None;
                        self.disk_images[slot] = None;
                    }
                    UiAction::LoadDisk { slot } => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Disk image", &["woz", "dsk", "po"])
                            .pick_file()
                            && let Some(disk) = read_disk(&path)
                            && let Some(card) =
                                self.apple2.peripheral_mut::<disk::ControllerCard>(slot)
                        {
                            insert_disk_bytes(card, &disk.data, disk.format);
                            self.disk_names[slot] =
                                path.file_name().map(|n| n.to_string_lossy().into_owned());
                            self.disk_images[slot] = Some(disk);
                        }
                    }
                }
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            self.display.draw(ui);
        });

        // Wake us up in time for the next emulated frame. While paused there's no
        // next frame, so we don't schedule a repaint: egui idles and repaints
        // only on interaction (which is how the Resume button stays responsive).
        if !self.paused {
            let wait = self.next_frame.saturating_duration_since(Instant::now());
            ctx.request_repaint_after(wait);
        }
    }
}
