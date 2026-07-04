use eframe::egui;

/// The number of peripheral slots exposed by the emulator.
const NUM_SLOTS: usize = 8;

/// A peripheral card the user can insert into a slot from the toolbar.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    LanguageCard,
    Serial,
    DiskController,
}

impl CardKind {
    /// Every card the user can choose from, in menu order.
    pub const ALL: [CardKind; 3] = [
        CardKind::LanguageCard,
        CardKind::Serial,
        CardKind::DiskController,
    ];

    /// Human-readable name shown in the menu.
    pub fn name(self) -> &'static str {
        match self {
            CardKind::LanguageCard => "Language Card",
            CardKind::Serial => "Super Serial Card",
            CardKind::DiskController => "Disk II Controller Card",
        }
    }
}

/// Where the Super Serial Card's host-side serial port comes from. Chosen by the
/// user at the moment the card is inserted.
#[derive(Clone, PartialEq, Eq)]
pub enum SerialPortSource {
    /// Spin up a virtual PTY pair.
    Pty,
    /// Open an existing serial device at the given path (e.g. `/dev/ttyUSB0`).
    Path(String),
}

/// User-configurable settings for the Super Serial Card, edited from its menu
/// and applied whenever the card is (re)built.
#[derive(Clone)]
pub struct SerialConfig {
    /// Where the host-side port comes from.
    pub source: SerialPortSource,
    /// Editing buffer for the "existing path" text field. Committed to `source`
    /// when the user connects; kept separate so typing doesn't churn the port.
    pub path_input: String,
    /// DIP switch bank 1 (bit 0 = SW1-1, ... bit 5 = SW1-6).
    pub sw1: u8,
    /// DIP switch bank 2 (bit 0 = SW2-1, ... bit 4 = SW2-5).
    pub sw2: u8,
}

impl Default for SerialConfig {
    fn default() -> Self {
        // Historical hard-coded ADTPro-friendly switch settings. `source` is a
        // placeholder; the real port is always chosen explicitly when the card
        // is inserted, so this value is never used to build a port on its own.
        SerialConfig {
            source: SerialPortSource::Pty,
            path_input: String::new(),
            sw1: 0b1111001,
            sw2: 0b0011011,
        }
    }
}

/// An action requested via the toolbar, executed by the app.
pub enum UiAction {
    Reset,
    /// Cold boot: power the machine off and back on (clears RAM, etc.).
    PowerCycle,
    /// Freeze/unfreeze emulation (the display keeps showing the last frame).
    TogglePause,
    /// Toggle 4x fast-forward emulation.
    ToggleFastForward,
    /// Silence/unsilence the emulated speaker without stopping emulation.
    ToggleMute,
    /// Prompt for a path and save the current screen as a PNG image.
    Screenshot,
    /// Insert (or replace) a card of `kind` into `slot`.
    InsertCard {
        slot: usize,
        kind: CardKind,
    },
    /// Remove whatever card currently occupies `slot`.
    RemoveCard {
        slot: usize,
    },
    /// Prompt for a disk image and load it into the Disk II controller in `slot`.
    LoadDisk {
        slot: usize,
    },
    /// Rebuild the Super Serial Card in `slot` from the current serial config
    /// (the config has already been mutated in place by the menu).
    ReconfigureSerial {
        slot: usize,
    },
}

/// Render the top toolbar. Returns the action the user triggered, if any.
///
/// This is intentionally free of any emulator or window state so the toolbar
/// stays a pure view: it only reports *what was clicked*, never acts on it.
/// `slots` is a read-only snapshot of which card occupies each slot so the menu
/// can label slots and highlight the active card. `disk_names` holds the name of
/// the disk image loaded into each slot's Drive 1 (if any), for display.
/// `paused` and `muted` reflect current state so the toggle buttons can show it.
/// `fast_forward` likewise reflects whether 4x emulation is active.
pub fn menu_bar(
    ui: &mut egui::Ui,
    slots: &[Option<CardKind>; NUM_SLOTS],
    disk_names: &[Option<String>; NUM_SLOTS],
    serial: &mut SerialConfig,
    paused: bool,
    fast_forward: bool,
    muted: bool,
) -> Option<UiAction> {
    let mut action = None;

    // Keep menus open while the user interacts with widgets inside them (text
    // fields, DIP switch checkboxes, port radios). The default `CloseOnClick`
    // closes the whole menu on any click inside it. Actions that should dismiss
    // the menu still call `ui.close()` explicitly.
    let config = egui::containers::menu::MenuConfig::new()
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);

    egui::containers::menu::MenuBar::new()
        .config(config)
        .ui(ui, |ui| {
            ui.menu_button("Slots", |ui| {
                // Don't wrap long card names; let the menu widen to fit them.
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                for (slot, current) in slots.iter().enumerate() {
                    let current = *current;
                    let label =
                        format!("Slot {slot}: {}", current.map_or("(Empty)", CardKind::name));

                    ui.menu_button(label, |ui| {
                        for kind in CardKind::ALL {
                            let active = current == Some(kind);

                            if active && kind == CardKind::DiskController {
                                // Active Disk II controller: hovering opens a drive
                                // submenu, while clicking the entry itself removes it.
                                let response = ui
                                    .menu_button(kind.name(), |ui| {
                                        ui.menu_button("Drive 1", |ui| {
                                            // Don't wrap long disk names; widen to fit.
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Extend);
                                            // Show the loaded image (or (Empty));
                                            // clicking it opens the load dialog.
                                            let label =
                                                disk_names[slot].as_deref().unwrap_or("(Empty)");
                                            if ui.button(label).clicked() {
                                                action = Some(UiAction::LoadDisk { slot });
                                                ui.close();
                                            }
                                        });
                                    })
                                    .response;

                                if response.clicked() {
                                    action = Some(UiAction::RemoveCard { slot });
                                    ui.close();
                                }
                            } else if active && kind == CardKind::Serial {
                                // Active Super Serial Card: hovering opens a config
                                // submenu (port + DIP switches), while clicking the
                                // entry itself removes the card.
                                let response = ui
                                    .menu_button(kind.name(), |ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                        serial_config_menu(ui, slot, serial, &mut action);
                                    })
                                    .response;

                                if response.clicked() {
                                    action = Some(UiAction::RemoveCard { slot });
                                    ui.close();
                                }
                            } else if kind == CardKind::Serial {
                                // Inserting a Super Serial Card requires choosing its
                                // port up front: the port choice is what inserts it.
                                ui.menu_button(kind.name(), |ui| {
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                    serial_insert_menu(ui, slot, serial, &mut action);
                                });
                            } else if ui.selectable_label(active, kind.name()).clicked() {
                                // Clicking the active card removes it; clicking any
                                // other card inserts (or replaces the occupant with) it.
                                action = Some(if active {
                                    UiAction::RemoveCard { slot }
                                } else {
                                    UiAction::InsertCard { slot, kind }
                                });
                                ui.close();
                            }
                        }
                    });
                }
            });

            // All action icons live on the far right, away from the config menus.
            // Added right-to-left, so the machine controls (reset, power cycle) sit
            // on the far edge, with a separator keeping them clear of the playback
            // toggles (pause, fast-forward, mute) to their left. Labels swap to
            // reflect current state; the F2/F3/F5/F6/F7 shortcuts are handled
            // host-side in `input`.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // NOTE: egui bundles its own fonts and only renders glyphs with
                // emoji presentation. The IEC power symbol (U+23FB) isn't an emoji
                // and shows as tofu, so use the plug emoji for power cycle instead.
                if ui
                    .button("\u{1F50C}")
                    .on_hover_text("Power Cycle (F3)")
                    .clicked()
                {
                    action = Some(UiAction::PowerCycle);
                }
                if ui.button("\u{1F504}").on_hover_text("Reset (F2)").clicked() {
                    action = Some(UiAction::Reset);
                }

                ui.separator();

                let (mute_icon, mute_hint) = if muted {
                    ("\u{1F507}", "Unmute (F7)")
                } else {
                    ("\u{1F50A}", "Mute (F7)")
                };
                if ui.button(mute_icon).on_hover_text(mute_hint).clicked() {
                    action = Some(UiAction::ToggleMute);
                }

                // Camera lives with the speaker in its own group, kept clear of
                // the playback toggles by a separator. U+1F4F7 has emoji
                // presentation so it renders in egui's bundled fonts.
                if ui
                    .button("\u{1F4F7}")
                    .on_hover_text("Screenshot (F12)")
                    .clicked()
                {
                    action = Some(UiAction::Screenshot);
                }

                ui.separator();

                // The playback toggles (fast-forward, pause) form their own group
                // to the left of the separator. Fast-forward stays highlighted
                // (`selected`) while active so the icon reflects whether we're
                // fast-forwarding without needing a second glyph.
                let ff_hint = if fast_forward {
                    "Normal Speed (F6)"
                } else {
                    "Fast Forward 4x (F6)"
                };
                if ui
                    .add(egui::Button::new("\u{23E9}").selected(fast_forward))
                    .on_hover_text(ff_hint)
                    .clicked()
                {
                    action = Some(UiAction::ToggleFastForward);
                }

                let (play_icon, play_hint) = if paused {
                    ("\u{25B6}", "Resume (F5)")
                } else {
                    ("\u{23F8}", "Pause (F5)")
                };
                if ui.button(play_icon).on_hover_text(play_hint).clicked() {
                    action = Some(UiAction::TogglePause);
                }
            });
        });

    action
}

/// Render the port-choice submenu shown when inserting a Super Serial Card.
///
/// The card isn't inserted until the user picks a port here: choosing "Virtual
/// (PTY)" or committing a device path sets `serial.source` and fires
/// [`UiAction::InsertCard`]. DIP switches keep their configured defaults and can
/// be tweaked afterwards from the active card's menu.
fn serial_insert_menu(
    ui: &mut egui::Ui,
    slot: usize,
    serial: &mut SerialConfig,
    action: &mut Option<UiAction>,
) {
    if ui.button("Virtual (PTY)").clicked() {
        serial.source = SerialPortSource::Pty;
        *action = Some(UiAction::InsertCard {
            slot,
            kind: CardKind::Serial,
        });
        ui.close();
    }

    ui.separator();
    ui.label("Existing path:");
    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut serial.path_input)
                .hint_text("/dev/ttyUSB0")
                .desired_width(140.0),
        );
        let submitted = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if (ui.button("Connect").clicked() || submitted) && !serial.path_input.is_empty() {
            serial.source = SerialPortSource::Path(serial.path_input.clone());
            *action = Some(UiAction::InsertCard {
                slot,
                kind: CardKind::Serial,
            });
            ui.close();
        }
    });
}

/// Render the Super Serial Card configuration submenu for the card in `slot`.
///
/// Mutates `serial` in place and, whenever a change requires the live card to be
/// rebuilt, sets `action` to [`UiAction::ReconfigureSerial`]. DIP switch toggles
/// apply immediately; the port path is only committed on connect so partial
/// typing doesn't repeatedly reopen the device.
fn serial_config_menu(
    ui: &mut egui::Ui,
    slot: usize,
    serial: &mut SerialConfig,
    action: &mut Option<UiAction>,
) {
    ui.menu_button("Port", |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        let is_pty = matches!(serial.source, SerialPortSource::Pty);
        if ui.radio(is_pty, "Virtual (PTY)").clicked() && !is_pty {
            serial.source = SerialPortSource::Pty;
            *action = Some(UiAction::ReconfigureSerial { slot });
        }

        let is_path = matches!(serial.source, SerialPortSource::Path(_));
        if ui.radio(is_path, "Existing path").clicked() && !is_path {
            // Switch the selection to path mode without reopening anything yet;
            // the actual connect happens via the button / Enter below.
            serial.source = SerialPortSource::Path(serial.path_input.clone());
        }

        ui.horizontal(|ui| {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut serial.path_input)
                    .hint_text("/dev/ttyUSB0")
                    .desired_width(140.0),
            );
            let submitted = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if (ui.button("Connect").clicked() || submitted) && !serial.path_input.is_empty() {
                serial.source = SerialPortSource::Path(serial.path_input.clone());
                *action = Some(UiAction::ReconfigureSerial { slot });
                ui.close();
            }
        });
    });

    ui.menu_button("DIP Switch 1", |ui| {
        for i in 0..6 {
            let mut on = (serial.sw1 >> i) & 1 == 1;
            if ui.checkbox(&mut on, format!("SW1-{}", i + 1)).changed() {
                serial.sw1 = (serial.sw1 & !(1 << i)) | (u8::from(on) << i);
                *action = Some(UiAction::ReconfigureSerial { slot });
            }
        }
    });

    ui.menu_button("DIP Switch 2", |ui| {
        for i in 0..5 {
            let mut on = (serial.sw2 >> i) & 1 == 1;
            if ui.checkbox(&mut on, format!("SW2-{}", i + 1)).changed() {
                serial.sw2 = (serial.sw2 & !(1 << i)) | (u8::from(on) << i);
                *action = Some(UiAction::ReconfigureSerial { slot });
            }
        }
    });
}
