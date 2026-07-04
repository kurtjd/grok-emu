use crate::audio::Machine;
use eframe::egui;

/// Carriage return — what the Apple II expects from the Return key.
const RETURN: u8 = b'\r';
/// ASCII backspace — the Apple II's delete/rubout code.
const BACKSPACE: u8 = 0x08;
/// ASCII escape — the Apple II's Esc key.
const ESCAPE: u8 = 0x1B;

/// Map an egui key to the *base* (unshifted) ASCII byte the core expects.
/// The core handles uppercasing and shift-symbol resolution itself, so we only
/// pass the raw key plus the shift/ctrl flags.
fn key_to_ascii(key: egui::Key) -> Option<u8> {
    use egui::Key::*;
    let c = match key {
        A => b'a',
        B => b'b',
        C => b'c',
        D => b'd',
        E => b'e',
        F => b'f',
        G => b'g',
        H => b'h',
        I => b'i',
        J => b'j',
        K => b'k',
        L => b'l',
        M => b'm',
        N => b'n',
        O => b'o',
        P => b'p',
        Q => b'q',
        R => b'r',
        S => b's',
        T => b't',
        U => b'u',
        V => b'v',
        W => b'w',
        X => b'x',
        Y => b'y',
        Z => b'z',
        Num0 => b'0',
        Num1 => b'1',
        Num2 => b'2',
        Num3 => b'3',
        Num4 => b'4',
        Num5 => b'5',
        Num6 => b'6',
        Num7 => b'7',
        Num8 => b'8',
        Num9 => b'9',
        Minus => b'-',
        Equals => b'=',
        OpenBracket => b'[',
        CloseBracket => b']',
        Semicolon => b';',
        Quote => b'\'',
        Comma => b',',
        Period => b'.',
        Slash => b'/',
        Backslash => b'\\',
        Space => b' ',
        Enter => RETURN,
        Backspace => BACKSPACE,
        _ => return None,
    };
    Some(c)
}

/// Translate this frame's egui keyboard events into emulator input.
///
/// Skipped while egui itself wants the keyboard (e.g. a focused text field) so
/// UI typing doesn't leak into the emulated machine.
///
/// Host-side actions requested via keyboard shortcuts that the app (not the
/// machine) must carry out. Menu shortcuts stay live even while paused.
#[derive(Default)]
pub struct HostRequests {
    /// F3 — power cycle (rebuilds the whole machine).
    pub power_cycle: bool,
    /// F5 — toggle pause.
    pub toggle_pause: bool,
    /// F6 — toggle 4x fast-forward.
    pub toggle_fast_forward: bool,
    /// F7 — toggle audio mute.
    pub toggle_mute: bool,
}

/// Translate this frame's egui keyboard events into emulator input, returning
/// any host-side shortcuts the caller must act on.
///
/// Skipped while egui itself wants the keyboard (e.g. a focused text field) so
/// UI typing doesn't leak into the emulated machine.
///
/// While `paused`, emulated keys are swallowed (not forwarded to the frozen
/// machine), but the host/menu shortcuts (F2 reset, F3 power cycle, F5 pause,
/// F6 fast-forward, F7 mute) stay live — otherwise F5 couldn't resume.
pub fn handle_input(machine: &mut Machine, ctx: &egui::Context, paused: bool) -> HostRequests {
    let mut requests = HostRequests::default();
    if ctx.egui_wants_keyboard_input() {
        return requests;
    }

    let events = ctx.input(|i| i.events.clone());
    for event in events {
        match event {
            // Printable characters, already shift/layout-resolved by egui. This
            // covers letters, digits, and shifted symbols (+, @, :, ?, ...) that
            // egui delivers as distinct logical keys we'd otherwise have to remap.
            egui::Event::Text(text) if !paused => {
                for ch in text.chars() {
                    if ch.is_ascii() {
                        machine.input(ch as u8, false, false);
                    }
                }
            }
            // Control keys (no Text is emitted for these) and Ctrl-combinations.
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => match key {
                // Host/menu shortcuts stay live regardless of pause state.
                // F2 is a host-side reset shortcut, not an emulated key.
                egui::Key::F2 => machine.reset(),
                // F3 is a host-side power-cycle shortcut, handled by the app.
                egui::Key::F3 => requests.power_cycle = true,
                egui::Key::F5 => requests.toggle_pause = true,
                egui::Key::F6 => requests.toggle_fast_forward = true,
                egui::Key::F7 => requests.toggle_mute = true,
                // Emulated keys are only forwarded while running.
                egui::Key::ArrowRight if !paused => machine.input_arrow(true),
                egui::Key::ArrowLeft if !paused => machine.input_arrow(false),
                egui::Key::Escape if !paused => machine.input(ESCAPE, false, false),
                egui::Key::Enter if !paused => machine.input(RETURN, false, false),
                egui::Key::Backspace if !paused => machine.input(BACKSPACE, false, false),
                // Printables come through Event::Text above; here we only forward
                // Ctrl+key (Text is suppressed while ctrl/cmd is held).
                _ if !paused && modifiers.ctrl => {
                    if let Some(ascii) = key_to_ascii(key) {
                        machine.input(ascii, false, true);
                    }
                }
                _ => {}
            },
            // egui swallows Ctrl+C/X/V into these before emitting a Key event, so
            // translate them back into emulator Ctrl-key presses (Ctrl+C is the
            // BASIC break, so this one actually matters).
            egui::Event::Copy if !paused => machine.input(b'c', false, true),
            egui::Event::Cut if !paused => machine.input(b'x', false, true),
            egui::Event::Paste(_) if !paused => machine.input(b'v', false, true),
            _ => {}
        }
    }

    requests
}
