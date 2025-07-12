//! Operations for properly setting status flags.
//!
//! Note: This is a direct port from my old Space Invaders emulator.
//! This can likely be rewritten to be cleaner, but status flags are very difficult to get right.
//! So just go with this for now.
use crate::{Cpu, Flags, BusHandler};

impl<T: BusHandler> Cpu<T> {
    /* Adds two values (plus the CY flag if wanted) and if the result is greater
     * than 8-bits than we know a carry out occurred.
     */
    pub(crate) fn update_flag_cy_add(&mut self, val1: u8, val2: u8, add_carry: bool) {
        let carry = (add_carry && self.flags.contains(Flags::CY)) as u16;
        let res = (val1 as u16).wrapping_add(val2 as u16).wrapping_add(carry) > 0xFF;
        self.flags.set(Flags::CY, res);
    }

    /* Adds the first four bits of two values (plus the CY flag if wanted) and
     * if the result is greater than 0xF we know a carry out of the first 4 bits
     * occurred.
     */
    pub(crate) fn update_flag_ac_add(&mut self, val1: u8, val2: u8, add_carry: bool) {
        let carry = (add_carry && self.flags.contains(Flags::CY)) as u8;
        let res = (val1 & 0xF).wrapping_add(val2 & 0xF).wrapping_add(carry) > 0xF;
        self.flags.set(Flags::AC, res);
    }

    /* Subtracts the second value from the first value by adding the two's
     * complement of the second value to the first (plus the CY flag if wanted)
     * and if the result is greater than 0xFF we know a borrow out occurred.
     */
    pub(crate) fn update_flag_cy_sub(&mut self, val1: u8, val2: u8, sub_borrow: bool) {
        let borrow = sub_borrow && self.flags.contains(Flags::CY);
        let res = (val1 as u16)
            .wrapping_add(!(val2 as u16))
            .wrapping_add(!borrow as u16)
            > 0xFF;
        self.flags.set(Flags::CY, res);
    }

    /* Subtracts the first four bits of the second value from the first four bits
     * of the first value by adding the two's complement of the second value to the
     * first (plus the CY flag if wanted) and if the result is greater than 0xF we
     * know a borrow out from the first 4 bits occurred.
     */
    pub(crate) fn update_flag_ac_sub(&mut self, val1: u8, val2: u8, sub_borrow: bool) {
        let borrow = sub_borrow && self.flags.contains(Flags::CY);
        let val1 = val1 & 0xF;
        let val2 = !val2 & 0xF;
        let res = val1.wrapping_add(val2).wrapping_add(!borrow as u8) > 0xF;
        self.flags.set(Flags::AC, res);
    }

    // Simply sets the CY and AC flags low.
    pub(crate) fn clear_cy_ac(&mut self) {
        self.flags.remove(Flags::CY | Flags::AC);
    }

    // Sets sign flag equal to value of most-significant bit (bit 7) of res.
    pub(crate) fn update_flag_s(&mut self, res: u8) {
        let res = (res & (1 << 7)) != 0;
        self.flags.set(Flags::S, res);
    }

    // Sets zero flag equal to 1 if res == 0, otherwise set to 0.
    pub(crate) fn update_flag_z(&mut self, res: u8) {
        self.flags.set(Flags::Z, res == 0);
    }

    // Sets parity flag to 1 if number of 1 bits in res is even, 0 otherwise.
    pub(crate) fn update_flag_p(&mut self, res: u8) {
        let res = (res.count_ones() % 2) == 0;
        self.flags.set(Flags::P, res);
    }

    // Called by add-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_add(&mut self, val1: u8, val2: u8, carry: bool) {
        let res = val1
            .wrapping_add(val2)
            .wrapping_add((carry && self.flags.contains(Flags::CY)) as u8);
        self.update_flag_z(res);
        self.update_flag_p(res);
        self.update_flag_s(res);
        self.update_flag_ac_add(val1, val2, carry);
        self.update_flag_cy_add(val1, val2, carry);
    }

    // Called by subract-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_sub(&mut self, val1: u8, val2: u8, carry: bool) {
        let res = val1
            .wrapping_sub(val2)
            .wrapping_sub((carry && self.flags.contains(Flags::CY)) as u8);
        self.update_flag_z(res);
        self.update_flag_p(res);
        self.update_flag_s(res);
        self.update_flag_ac_sub(val1, val2, carry);
        self.update_flag_cy_sub(val1, val2, carry);
    }

    // Called by inc-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_inc(&mut self, val: u8) {
        let res = val.wrapping_add(1);
        self.update_flag_z(res);
        self.update_flag_p(res);
        self.update_flag_s(res);
        self.update_flag_ac_add(val, 1, false);
    }

    // Called by dec-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_dec(&mut self, val: u8) {
        let res = val.wrapping_sub(1);
        self.update_flag_z(res);
        self.update_flag_p(res);
        self.update_flag_s(res);
        self.update_flag_ac_sub(val, 1, false);
    }

    // Called by logical-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_log(&mut self, val: u8) {
        self.update_flag_z(val);
        self.update_flag_p(val);
        self.update_flag_s(val);
    }

    // Called by and-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_and(&mut self, val1: u8, val2: u8) {
        self.update_flags_log(val1 & val2);
        self.flags.remove(Flags::CY);

        // Weird note: Logical AND ops set AC to the logical or of bit 3
        self.flags.set(Flags::AC, (((val1 | val2) >> 3) & 1) == 1);
    }

    // Called by or/xor-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_or(&mut self, val: u8) {
        self.update_flags_log(val);
        self.clear_cy_ac();
    }

    // Called by cmp-related opcodes that follow standard flag update behavior.
    pub(crate) fn update_flags_cmp(&mut self, val1: u8, val2: u8) {
        let res = val1.wrapping_sub(val2);
        self.update_flag_z(res);
        self.update_flag_p(res);
        self.update_flag_s(res);
        self.update_flag_ac_sub(val1, val2, false);
        self.update_flag_cy_sub(val1, val2, false);
    }
}
