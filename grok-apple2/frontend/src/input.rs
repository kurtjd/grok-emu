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
/// Returns `true` if the user pressed the power-cycle shortcut (F3), which the
/// caller must act on since it rebuilds the whole machine, not just poke it.
pub fn handle_input(machine: &mut Machine, ctx: &egui::Context) -> bool {
    if ctx.egui_wants_keyboard_input() {
        return false;
    }

    let mut power_cycle = false;
    let events = ctx.input(|i| i.events.clone());
    for event in events {
        match event {
            // Printable characters, already shift/layout-resolved by egui. This
            // covers letters, digits, and shifted symbols (+, @, :, ?, ...) that
            // egui delivers as distinct logical keys we'd otherwise have to remap.
            egui::Event::Text(text) => {
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
                // F2 is a host-side reset shortcut, not an emulated key.
                egui::Key::F2 => machine.reset(),
                // F3 is a host-side power-cycle shortcut, handled by the app.
                egui::Key::F3 => power_cycle = true,
                egui::Key::ArrowRight => machine.input_arrow(true),
                egui::Key::ArrowLeft => machine.input_arrow(false),
                egui::Key::Escape => machine.input(ESCAPE, false, false),
                egui::Key::Enter => machine.input(RETURN, false, false),
                egui::Key::Backspace => machine.input(BACKSPACE, false, false),
                // Printables come through Event::Text above; here we only forward
                // Ctrl+key (Text is suppressed while ctrl/cmd is held).
                _ if modifiers.ctrl => {
                    if let Some(ascii) = key_to_ascii(key) {
                        machine.input(ascii, false, true);
                    }
                }
                _ => {}
            },
            // egui swallows Ctrl+C/X/V into these before emitting a Key event, so
            // translate them back into emulator Ctrl-key presses (Ctrl+C is the
            // BASIC break, so this one actually matters).
            egui::Event::Copy => machine.input(b'c', false, true),
            egui::Event::Cut => machine.input(b'x', false, true),
            egui::Event::Paste(_) => machine.input(b'v', false, true),
            _ => {}
        }
    }

    power_cycle
}
