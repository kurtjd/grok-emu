pub trait BusHandlerZ80 {
    fn addr(&self) -> u16;
    fn set_addr(&mut self, val: u16);

    fn data(&self) -> u8;
    fn set_data(&mut self, val: u8);

    fn busack(&self) -> bool;
    fn set_busack(&mut self, val: bool);

    fn busreq(&self) -> bool;
    fn set_busreq(&mut self, val: bool);

    fn iorq(&self) -> bool;
    fn set_iorq(&mut self, val: bool);

    fn m1(&self) -> bool;
    fn set_m1(&mut self, val: bool);

    fn mreq(&self) -> bool;
    fn set_mreq(&mut self, val: bool);

    fn rd(&self) -> bool;
    fn set_rd(&mut self, val: bool);

    fn wr(&self) -> bool;
    fn set_wr(&mut self, val: bool);

    fn int(&self) -> bool;
    fn set_int(&mut self, val: bool);

    fn nmi(&self) -> bool;
    fn set_nmi(&mut self, val: bool);

    fn rfsh(&self) -> bool;
    fn set_rfsh(&mut self, val: bool);

    fn wait(&self) -> bool;
    fn set_wait(&mut self, val: bool);

    fn reset(&self) -> bool;
    fn set_reset(&mut self, val: bool);

    fn halt(&self) -> bool;
    fn set_halt(&mut self, val: bool);
}

#[derive(Default)]
pub struct BusZ80 {
    addr: u16,
    data: u8,
    busack: bool,
    busreq: bool,
    iorq: bool,
    m1: bool,
    mreq: bool,
    rd: bool,
    wr: bool,
    int: bool,
    nmi: bool,
    rfsh: bool,
    wait: bool,
    reset: bool,
    halt: bool,
}

impl BusZ80 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BusHandlerZ80 for BusZ80 {
    fn addr(&self) -> u16 {
        self.addr
    }

    fn set_addr(&mut self, val: u16) {
        self.addr = val
    }

    fn data(&self) -> u8 {
        self.data
    }

    fn set_data(&mut self, val: u8) {
        self.data = val
    }

    fn busack(&self) -> bool {
        self.busack
    }

    fn set_busack(&mut self, val: bool) {
        self.busack = val
    }

    fn busreq(&self) -> bool {
        self.busreq
    }

    fn set_busreq(&mut self, val: bool) {
        self.busreq = val
    }

    fn iorq(&self) -> bool {
        self.iorq
    }

    fn set_iorq(&mut self, val: bool) {
        self.iorq = val
    }

    fn m1(&self) -> bool {
        self.m1
    }

    fn set_m1(&mut self, val: bool) {
        self.m1 = val
    }

    fn mreq(&self) -> bool {
        self.mreq
    }

    fn set_mreq(&mut self, val: bool) {
        self.mreq = val
    }

    fn rd(&self) -> bool {
        self.rd
    }

    fn set_rd(&mut self, val: bool) {
        self.rd = val
    }

    fn wr(&self) -> bool {
        self.wr
    }

    fn set_wr(&mut self, val: bool) {
        self.wr = val
    }

    fn int(&self) -> bool {
        self.int
    }

    fn set_int(&mut self, val: bool) {
        self.int = val
    }

    fn nmi(&self) -> bool {
        self.nmi
    }

    fn set_nmi(&mut self, val: bool) {
        self.nmi = val
    }

    fn rfsh(&self) -> bool {
        self.rfsh
    }

    fn set_rfsh(&mut self, val: bool) {
        self.rfsh = val
    }

    fn wait(&self) -> bool {
        self.wait
    }

    fn set_wait(&mut self, val: bool) {
        self.wait = val
    }

    fn reset(&self) -> bool {
        self.reset
    }

    fn set_reset(&mut self, val: bool) {
        self.reset = val
    }

    fn halt(&self) -> bool {
        self.halt
    }

    fn set_halt(&mut self, val: bool) {
        self.halt = val
    }
}
