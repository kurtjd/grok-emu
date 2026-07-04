use crate::audio::{self, CpalAudio, Machine};
use crate::gui::{self, CardKind, ScreenAspect, SerialConfig, SerialPortSource, UiAction};
use crate::input;
use crate::serial::StdSerialPort;
use crate::video::Display;
use eframe::egui;
use grok_apple2_core::peripheral::serial::SuperSerial;
use grok_apple2_core::peripheral::{Peripheral, disk, language};
use grok_apple2_core::{Apple2, settings};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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

// Emulation speed multiplier applied while fast-forwarding: the number of frames
// run per displayed frame, and the audio sink's sample stride (see `CpalAudio`).
const FAST_FORWARD_FACTOR: u32 = 4;

// Fullscreen overlay timing. The floating toolbar reveals on mouse movement and
// auto-hides after `FS_REVEAL_SECS` of stillness, fading over `FS_FADE_SECS`. The
// "how to exit" hint holds for `FS_TOAST_HOLD` then fades over `FS_TOAST_FADE`.
const FS_REVEAL_SECS: f32 = 2.0;
const FS_FADE_SECS: f32 = 0.25;
const FS_TOAST_HOLD: f32 = 2.5;
const FS_TOAST_FADE: f32 = 0.6;

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
    // Emulation speed factor (1 = normal, N = Nx fast-forward): the number of
    // `run_frame` calls per displayed frame. Shared with the audio sink so it can
    // decimate samples to match (sped-up "chipmunk" audio), so it must be
    // reattached to the rebuilt sink on a power cycle.
    speed: std::sync::Arc<AtomicU32>,
    // When true, emulation is frozen: `run_frame` is skipped so the display holds
    // the last frame and the audio ring drains to silence.
    paused: bool,
    // How the emulated display is fitted to the window (square pixels vs 4:3).
    aspect: ScreenAspect,
    // Fullscreen overlay bookkeeping: whether we were fullscreen last frame (to
    // detect entering it) and when we entered (to time the hint toast).
    was_fullscreen: bool,
    fullscreen_entered_at: Option<Instant>,
    // Held to keep the audio stream alive for the lifetime of the app.
    _audio_stream: cpal::Stream,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (audio, audio_stream, audio_muted, speed) = audio::init_audio();
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
            speed,
            paused: false,
            aspect: ScreenAspect::default(),
            was_fullscreen: false,
            fullscreen_entered_at: None,
            _audio_stream: audio_stream,
        }
    }

    /// Simulate turning the Apple II off and back on: tear down the running
    /// machine and build a brand-new one, giving cleared RAM, reset soft
    /// switches, reset peripheral state, and a fresh CPU. Inserted disks are
    /// re-inserted (the floppy stays in the drive across a power cycle); the
    /// audio stream is reattached via the shared ring buffer.
    fn power_cycle(&mut self) {
        let audio = CpalAudio::new(self.audio_ring.clone(), self.speed.clone());
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

    /// Toggle fast-forward on/off. The speed factor is the single source of
    /// truth: it drives both how many frames run per tick and the stride the
    /// audio sink uses to keep playback in sync.
    fn toggle_fast_forward(&mut self) {
        let factor = if self.speed.load(Ordering::Relaxed) == 1 {
            FAST_FORWARD_FACTOR
        } else {
            1
        };
        self.speed.store(factor, Ordering::Relaxed);
    }

    /// Prompt for a destination and save the currently displayed frame as a PNG.
    /// Cancelling the dialog is a no-op; write errors are logged, not fatal.
    fn save_screenshot(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("apple2-screenshot.png")
            .save_file()
        {
            self.display.save_png(&path);
        }
    }

    /// Render the toolbar contents (menu bar + action buttons) into `ui`, returning
    /// the action the user triggered. Shared by the docked (windowed) toolbar and
    /// the floating fullscreen overlay.
    fn toolbar_ui(&mut self, ui: &mut egui::Ui, fullscreen: bool) -> Option<UiAction> {
        gui::menu_bar(
            ui,
            &self.slots,
            &self.disk_names,
            &mut self.serial,
            &mut self.aspect,
            gui::ToolbarState {
                paused: self.paused,
                fast_forward: self.speed.load(Ordering::Relaxed) != 1,
                muted: self.audio_muted.load(Ordering::Relaxed),
                fullscreen,
            },
        )
    }

    /// Execute a toolbar action. `fullscreen` is the current window state, used to
    /// flip it on a fullscreen toggle.
    fn apply_action(&mut self, action: UiAction, ctx: &egui::Context, fullscreen: bool) {
        match action {
            UiAction::Reset => self.apple2.reset(),
            UiAction::PowerCycle => self.power_cycle(),
            UiAction::TogglePause => self.set_paused(!self.paused),
            UiAction::ToggleFastForward => self.toggle_fast_forward(),
            UiAction::ToggleFullscreen => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
            }
            UiAction::ToggleMute => {
                self.audio_muted.fetch_xor(true, Ordering::Relaxed);
            }
            UiAction::Screenshot => self.save_screenshot(),
            UiAction::InsertCard { slot, kind } => {
                self.apple2
                    .insert_peripheral(build_card(kind, &self.serial), slot);
                self.slots[slot] = Some(kind);
                self.disk_names[slot] = None;
                self.disk_images[slot] = None;
            }
            UiAction::ReconfigureSerial { slot } => {
                // Config was already updated in place by the menu; tear down and
                // rebuild the card so the change takes effect.
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
                    && let Some(card) = self.apple2.peripheral_mut::<disk::ControllerCard>(slot)
                {
                    insert_disk_bytes(card, &disk.data, disk.format);
                    self.disk_names[slot] =
                        path.file_name().map(|n| n.to_string_lossy().into_owned());
                    self.disk_images[slot] = Some(disk);
                }
            }
        }
    }

    /// Render the fullscreen toolbar as a floating overlay that fades in on mouse
    /// activity and auto-hides (with the OS cursor) after a short idle, so it never
    /// reflows the display the way a docked panel would.
    fn fullscreen_overlay(&mut self, ctx: &egui::Context) {
        let idle = ctx.input(|i| i.pointer.time_since_last_movement());
        let show = idle < FS_REVEAL_SECS;
        let alpha =
            ctx.animate_bool_with_time(egui::Id::new("fs_toolbar_fade"), show, FS_FADE_SECS);

        if alpha > 0.0 {
            let width = ctx.content_rect().width();
            let action = egui::Area::new(egui::Id::new("fs_toolbar"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_opacity(alpha);
                    egui::Frame::new()
                        .fill(egui::Color32::from_black_alpha(200))
                        .inner_margin(egui::Margin::symmetric(6, 4))
                        .show(ui, |ui| {
                            ui.set_width(width - 12.0);
                            self.toolbar_ui(ui, true)
                        })
                        .inner
                })
                .inner;
            if let Some(action) = action {
                self.apply_action(action, ctx, true);
            }
        } else {
            // Fully idle: hide the cursor for a clean, immersive picture.
            ctx.set_cursor_icon(egui::CursorIcon::None);
        }

        self.fullscreen_hint(ctx);
    }

    /// Show a fading "how to get out" hint for a moment after entering fullscreen,
    /// then forget it.
    fn fullscreen_hint(&mut self, ctx: &egui::Context) {
        let Some(entered) = self.fullscreen_entered_at else {
            return;
        };
        let elapsed = entered.elapsed().as_secs_f32();
        let alpha = if elapsed < FS_TOAST_HOLD {
            1.0
        } else {
            (1.0 - (elapsed - FS_TOAST_HOLD) / FS_TOAST_FADE).clamp(0.0, 1.0)
        };
        if alpha <= 0.0 {
            self.fullscreen_entered_at = None;
            return;
        }

        egui::Area::new(egui::Id::new("fs_hint"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 48.0))
            .show(ctx, |ui| {
                ui.set_opacity(alpha);
                egui::Frame::new()
                    .fill(egui::Color32::from_black_alpha(200))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                "Move mouse to top for menu   \u{2022}   F11 to exit",
                            )
                            .color(egui::Color32::WHITE),
                        );
                    });
            });
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
        // Current fullscreen state from the windowing system, so the toggle and
        // its menu checkmark stay in sync with reality rather than a tracked flag.
        let fullscreen = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
        // Detect entering fullscreen (to time the hint toast); clear it on exit.
        if fullscreen && !self.was_fullscreen {
            self.fullscreen_entered_at = Some(Instant::now());
        } else if !fullscreen {
            self.fullscreen_entered_at = None;
        }
        self.was_fullscreen = fullscreen;

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
        if requests.toggle_fast_forward {
            self.toggle_fast_forward();
        }
        if requests.toggle_mute {
            self.audio_muted.fetch_xor(true, Ordering::Relaxed);
        }
        if requests.take_screenshot {
            self.save_screenshot();
        }
        if requests.toggle_fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }

        // Advance the emulation at a fixed 60 Hz regardless of how often eframe
        // repaints (input events can trigger extra repaints). Skipped while
        // paused so the machine freezes on the current frame. While
        // fast-forwarding, run several frames per tick so emulated time advances
        // faster; only the last frame is displayed.
        if !self.paused && Instant::now() >= self.next_frame {
            let steps = self.speed.load(Ordering::Relaxed).max(1);
            // Run all but the last frame purely to advance state; each
            // `run_frame` borrows the machine, so we can't keep the earlier
            // slices around — only the final frame is drawn.
            for _ in 1..steps {
                self.apple2.run_frame();
            }
            let frame = self.apple2.run_frame();
            self.display.update(frame);

            self.next_frame += FRAME_TIME;
            // If we've fallen behind, resync rather than trying to catch up forever.
            let now = Instant::now();
            if self.next_frame < now {
                self.next_frame = now + FRAME_TIME;
            }
        }

        if fullscreen {
            self.fullscreen_overlay(&ctx);
        } else {
            let action = egui::Panel::top("toolbar")
                .show(ui, |ui| self.toolbar_ui(ui, fullscreen))
                .inner;
            if let Some(action) = action {
                self.apply_action(action, &ctx, fullscreen);
            }
        }

        // Apply the chosen aspect/filtering. Cheap when unchanged, and re-renders
        // the buffered frame on change so it updates instantly even while paused.
        self.display.set_aspect(self.aspect);

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

        // In fullscreen, keep repainting while the overlay could still be animating
        // toward hidden (or the hint is up), so the auto-hide and cursor-hide fire
        // even when emulation is paused.
        if fullscreen {
            let idle = ctx.input(|i| i.pointer.time_since_last_movement());
            if idle < FS_REVEAL_SECS + FS_FADE_SECS + 0.1 || self.fullscreen_entered_at.is_some() {
                ctx.request_repaint();
            }
        }
    }
}
