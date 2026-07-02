use grok_6502::bus::{self, Bus};

const DATA: u16 = 0;
const CLEAR: u16 = 1;
const KEY_RIGHT: u8 = 0x95;
const KEY_LEFT: u8 = 0x88;

pub(crate) struct Keyboard {
    data: u8,
}

impl Keyboard {
    pub(crate) fn new() -> Self {
        Keyboard { data: 0 }
    }

    pub(crate) fn input(&mut self, char: u8, shift: bool, ctrl: bool) {
        // Convert lowercase to uppercase
        let mut ascii = char;
        if ascii.is_ascii_lowercase() {
            ascii -= 32;
        }

        // Get the proper ASCII character if shift held
        if shift {
            ascii = self.get_shift_ascii(ascii);
        }

        // Do nothing if not a valid Apple 2 key
        if !self.is_valid_key(ascii) {
            return;
        }

        // Modify the value (if necessary) when CTRL is held
        if ctrl {
            ascii = self.get_ctrl_ascii(ascii);
        }

        // The Apple 2 has the high bit set for ASCII characters
        self.data = ascii | (1 << 7);
    }

    pub(crate) fn input_arrow(&mut self, right: bool) {
        let ascii = if right { KEY_RIGHT } else { KEY_LEFT };
        self.data = ascii;
    }

    pub(crate) fn decode(&mut self, bus: &mut dyn Bus) {
        match (bus.addr() >> 4) & 0xF {
            DATA if bus.op() == bus::Op::Read => bus.set_data(self.data),
            CLEAR => self.data &= !(1 << 7),
            _ => (),
        }
    }

    fn is_valid_key(&self, ascii: u8) -> bool {
        // 8 = ASCII for backspace, 13 = ASCII for return/enter
        matches!(ascii, b' '..=b'^' | b'_' | 8 | 13)
    }

    fn get_shift_ascii(&self, ascii: u8) -> u8 {
        match ascii {
            b'1' => b'!',
            b'2' => b'@',
            b'3' => b'#',
            b'4' => b'$',
            b'5' => b'%',
            b'6' => b'^',
            b'7' => b'&',
            b'8' => b'*',
            b'9' => b'(',
            b'0' => b')',
            b'-' => b'_',
            b'=' => b'+',
            b'[' => b'{',
            b']' => b'}',
            b';' => b':',
            b'\'' => b'"',
            b',' => b'<',
            b'.' => b'>',
            b'/' => b'?',
            _ => ascii,
        }
    }

    fn get_ctrl_ascii(&self, ascii: u8) -> u8 {
        // Ctrl only modified A-Z keys by clearing the 6th bit
        match ascii.is_ascii_uppercase() {
            true => ascii & !(1 << 6),
            false => ascii,
        }
    }
}
