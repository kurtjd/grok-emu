//! Operations for properly setting status flags.
//!
//! Note: This is a direct port from my old Space Invaders emulator.
//! This can likely be rewritten to be cleaner, but status flags are very difficult to get right.
//! So just go with this for now.
use crate::{BusHandlerZ80, Cpu, Flags};

impl<T: BusHandlerZ80> Cpu<T> {
    /* Adds two values (plus the C flag if wanted) and if the result is greater
     * than 8-bits than we know a carry out occurred.
     */
    pub(crate) fn update_flag_c_add(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = (val1 as u16) + (val2 as u16) + (carry as u16) > 0xFF;
        self.reg.f.set(Flags::C, res);
    }

    /* Adds the first four bits of two values (plus the C flag if wanted) and
     * if the result is greater than 0xF we know a carry out of the first 4 bits
     * occurred.
     */
    pub(crate) fn update_flag_h_add(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = (val1 & 0xF) + (val2 & 0xF) + carry > 0xF;
        self.reg.f.set(Flags::H, res);
    }

    /* Subtracts the second value from the first value by adding the two's
     * complement of the second value to the first (plus the C flag if wanted)
     * and if the result is greater than 8-bits we know a borrow out occurred.
     */
    pub(crate) fn update_flag_c_sub(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = (val1 as i16) - (val2 as i16) - (carry as i16) < 0x00;
        self.reg.f.set(Flags::C, res);
    }

    /* Subtracts the first four bits of the second value from the first four bits
     * of the first value by adding the two's complement of the second value to the
     * first (plus the C flag if wanted) and if the result is greater than 0xF we
     * know a borrow out from the first 4 bits occurred.
     */
    pub(crate) fn update_flag_h_sub(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = (val1 & 0xF) as i8 - (val2 & 0xF) as i8 - (carry as i8) < 0x0;
        self.reg.f.set(Flags::H, res);
    }

    // Sets sign flag equal to value of most-significant bit (bit 7) of res.
    pub(crate) fn update_flag_s(&mut self, res: u8) {
        let res = (res & (1 << 7)) != 0;
        self.reg.f.set(Flags::S, res);
    }

    // Sets zero flag equal to 1 if res == 0, otherwise set to 0.
    pub(crate) fn update_flag_z(&mut self, res: u8) {
        self.reg.f.set(Flags::Z, res == 0);
    }

    // Sets parity flag to 1 if number of 1 bits in res is even, 0 otherwise.
    pub(crate) fn update_flag_p(&mut self, res: u8) {
        let res = (res.count_ones() % 2) == 0;
        self.reg.f.set(Flags::P, res);
    }

    // Updates flags Z, P, and S according to typical behavior.
    pub(crate) fn update_flags_zps(&mut self, val: u8) {
        self.update_flag_z(val);
        self.update_flag_p(val);
        self.update_flag_s(val);
    }

    // Sets parity flag to 1 if signed overflow.
    pub(crate) fn update_flag_pv_add(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = val1 + val2 + carry;
        let res = (!(val1 ^ val2) & (val1 ^ res)) & (1 << 7) != 0;
        self.reg.f.set(Flags::P, res);
    }

    // Sets parity flag to 1 if signed overflow.
    pub(crate) fn update_flag_pv_sub(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = val1 - val2 - carry;
        let res = ((val1 ^ val2) & (val1 ^ res)) & (1 << 7) != 0;
        self.reg.f.set(Flags::P, res);
    }

    // Called by add-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_add(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = val1 + val2 + carry;
        self.reg.f.remove(Flags::N);
        self.update_flag_z(res);
        self.update_flag_s(res);
        self.update_flag_pv_add(val1, val2, carry);
        self.update_flag_h_add(val1, val2, carry);
        self.update_flag_c_add(val1, val2, carry);
    }

    // Called by subract-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_sub(&mut self, val1: u8, val2: u8, carry: u8) {
        let res = val1 - val2 - carry;
        self.reg.f.insert(Flags::N);
        self.update_flag_z(res);
        self.update_flag_s(res);
        self.update_flag_pv_sub(val1, val2, carry);
        self.update_flag_h_sub(val1, val2, carry);
        self.update_flag_c_sub(val1, val2, carry);
    }

    // Called by inc-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_inc(&mut self, val: u8) {
        let res = val + 1;
        self.update_flag_z(res);
        self.update_flag_s(res);
        self.update_flag_h_add(val, 1, 0);
        self.reg.f.remove(Flags::N);
        self.reg.f.set(Flags::P, val == 0x7F);
    }

    // Called by dec-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_dec(&mut self, val: u8) {
        let res = val - 1;
        self.update_flag_z(res);
        self.update_flag_s(res);
        self.update_flag_h_sub(val, 1, 0);
        self.reg.f.insert(Flags::N);
        self.reg.f.set(Flags::P, val == 0x80);
    }

    // Called by and-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_and(&mut self, val: u8) {
        self.update_flags_zps(val);
        self.reg.f.remove(Flags::C | Flags::N);
        self.reg.f.insert(Flags::H);
    }

    // Called by or/xor-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_or(&mut self, val: u8) {
        self.update_flags_zps(val);
        self.reg.f.remove(Flags::C | Flags::N | Flags::H);
    }

    // Called by cmp-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_cmp(&mut self, val1: u8, val2: u8) {
        let res = val1 - val2;
        self.update_flag_z(res);
        self.update_flag_s(res);
        self.update_flag_pv_sub(val1, val2, 0);
        self.update_flag_h_sub(val1, val2, 0);
        self.update_flag_c_sub(val1, val2, 0);
        self.reg.f.insert(Flags::N);
    }

    // Called by all instructions that modify flags
    pub(crate) fn update_x_y_q(&mut self, val: u8) {
        self.reg.f.set(Flags::X, val & Flags::X.bits() != 0);
        self.reg.f.set(Flags::Y, val & Flags::Y.bits() != 0);
        self.reg.q = self.reg.f.bits();
    }

    // Called by a few instructions that have alternate X/Y behavior
    // In these, X/Y are dependent on the value of Q
    pub(crate) fn update_x_y_q_alt(&mut self, val: u8) {
        self.reg.f.set(
            Flags::X,
            (val | (self.reg.q ^ self.reg.f.bits())) & Flags::X.bits() != 0,
        );
        self.reg.f.set(
            Flags::Y,
            (val | (self.reg.q ^ self.reg.f.bits())) & Flags::Y.bits() != 0,
        );
        self.reg.q = self.reg.f.bits();
    }
}
