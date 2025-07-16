//! Emulation of the sound registers used by Space Invaders

bitflags::bitflags! {
    struct RegFlags1: u8 {
        const UFO = 1 << 0;
        const SHOOT = 1 << 1;
        const PLAYER_DIE = 1 << 2;
        const INVADER_DIE = 1 << 3;
        const EXTRA_LIFE = 1 << 4;
    }
}

bitflags::bitflags! {
    struct RegFlags2: u8 {
        const FLEET_MOVE1 = 1 << 0;
        const FLEET_MOVE2 = 1 << 1;
        const FLEET_MOVE3 = 1 << 2;
        const FLEET_MOVE4 = 1 << 3;
        const UFO_DIE = 1 << 4;
    }
}

pub(crate) struct SoundReg {
    reg1: RegFlags1,
    reg2: RegFlags2,
}

impl SoundReg {
    pub(crate) fn new() -> Self {
        Self {
            reg1: RegFlags1::empty(),
            reg2: RegFlags2::empty(),
        }
    }

    pub(crate) fn set_reg1(&mut self, val: u8) {
        self.reg1 = RegFlags1::from_bits_truncate(val);
    }

    pub(crate) fn set_reg2(&mut self, val: u8) {
        self.reg2 = RegFlags2::from_bits_truncate(val);
    }

    pub(crate) fn ufo(&self) -> bool {
        self.reg1.contains(RegFlags1::UFO)
    }

    pub(crate) fn shoot(&self) -> bool {
        self.reg1.contains(RegFlags1::SHOOT)
    }

    pub(crate) fn player_die(&self) -> bool {
        self.reg1.contains(RegFlags1::PLAYER_DIE)
    }

    pub(crate) fn invader_die(&self) -> bool {
        self.reg1.contains(RegFlags1::INVADER_DIE)
    }

    pub(crate) fn extra_life(&self) -> bool {
        self.reg1.contains(RegFlags1::EXTRA_LIFE)
    }

    pub(crate) fn fleet_move1(&self) -> bool {
        self.reg2.contains(RegFlags2::FLEET_MOVE1)
    }

    pub(crate) fn fleet_move2(&self) -> bool {
        self.reg2.contains(RegFlags2::FLEET_MOVE2)
    }

    pub(crate) fn fleet_move3(&self) -> bool {
        self.reg2.contains(RegFlags2::FLEET_MOVE3)
    }

    pub(crate) fn fleet_move4(&self) -> bool {
        self.reg2.contains(RegFlags2::FLEET_MOVE4)
    }

    pub(crate) fn ufo_die(&self) -> bool {
        self.reg2.contains(RegFlags2::UFO_DIE)
    }
}
