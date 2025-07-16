//! Emulation of the input registers used by Space Invaders

bitflags::bitflags! {
    struct RegFlags1: u8 {
        const CREDIT = 1 << 0;
        const START_2P = 1 << 1;
        const START_1P = 1 << 2;
        const SHOOT_P1 = 1 << 4;
        const LEFT_P1 = 1 << 5;
        const RIGHT_P1 = 1 << 6;
    }
}

bitflags::bitflags! {
    struct RegFlags2: u8 {
        const TILT = 1 << 2;
        const SHOOT_P2 = 1 << 4;
        const LEFT_P2 = 1 << 5;
        const RIGHT_P2 = 1 << 6;
    }
}

pub(crate) struct InputReg {
    reg1: RegFlags1,
    reg2: RegFlags2,
}

impl InputReg {
    pub(crate) fn new() -> Self {
        Self {
            reg1: RegFlags1::empty(),
            reg2: RegFlags2::empty(),
        }
    }

    pub(crate) fn read_reg1(&self) -> u8 {
        self.reg1.bits()
    }

    pub(crate) fn read_reg2(&self) -> u8 {
        self.reg2.bits()
    }

    pub(crate) fn set_credit(&mut self, val: bool) {
        self.reg1.set(RegFlags1::CREDIT, val)
    }

    pub(crate) fn set_start_2p(&mut self, val: bool) {
        self.reg1.set(RegFlags1::START_2P, val)
    }

    pub(crate) fn set_start_1p(&mut self, val: bool) {
        self.reg1.set(RegFlags1::START_1P, val)
    }

    pub(crate) fn set_shoot_p1(&mut self, val: bool) {
        self.reg1.set(RegFlags1::SHOOT_P1, val)
    }

    pub(crate) fn set_left_p1(&mut self, val: bool) {
        self.reg1.set(RegFlags1::LEFT_P1, val)
    }

    pub(crate) fn set_right_p1(&mut self, val: bool) {
        self.reg1.set(RegFlags1::LEFT_P1, val)
    }

    pub(crate) fn set_tilt(&mut self, val: bool) {
        self.reg2.set(RegFlags2::TILT, val)
    }

    pub(crate) fn set_shoot_p2(&mut self, val: bool) {
        self.reg2.set(RegFlags2::SHOOT_P2, val)
    }

    pub(crate) fn set_left_p2(&mut self, val: bool) {
        self.reg2.set(RegFlags2::LEFT_P2, val)
    }

    pub(crate) fn set_right_p2(&mut self, val: bool) {
        self.reg2.set(RegFlags2::LEFT_P2, val)
    }
}
