use grok_6502::bus::{Bus, SimpleBus};

const INPUT_DATA: usize = 0xC000;
const INPUT_CLEAR: u16 = 0xC010; // Whole page

pub(crate) const KEY_RIGHT: u8 = 0x95;
pub(crate) const KEY_LEFT: u8 = 0x88;
pub(crate) const DATA_ADDR: usize = 0xC000;

pub(crate) fn is_valid_key(ascii: u8) -> bool {
    // 8 = ASCII for backspace, 13 = ASCII for return/enter
    matches!(ascii, b' '..=b'^' | b'_' | 8 | 13)
}

pub(crate) fn get_shift_ascii(ascii: u8) -> u8 {
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

pub(crate) fn get_ctrl_ascii(ascii: u8) -> u8 {
    // Ctrl only modified A-Z keys by clearing the 6th bit
    match ascii.is_ascii_uppercase() {
        true => ascii & !(1 << 6),
        false => ascii,
    }
}

pub(crate) fn tick(bus: &mut SimpleBus, memory: &mut [u8]) {
    if bus.addr() == INPUT_CLEAR {
        memory[INPUT_DATA] &= !(1 << 7);
    }
}
