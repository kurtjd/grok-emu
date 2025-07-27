use crate::Cpu;
use grok_bus::BusHandlerZ80;

impl<B: BusHandlerZ80> Cpu<B> {
    pub(crate) fn fetch_t1(&mut self, bus: &mut B) {
        bus.set_addr(self.reg.spr.pc);
        bus.set_m1(true);
        bus.set_rfsh(false);
        self.reg.spr.pc += 1;
    }

    pub(crate) fn fetch_t2(&mut self, bus: &mut B) {
        bus.set_mreq(true);
        bus.set_rd(true);
    }

    pub(crate) fn fetch_t3(&mut self, bus: &mut B) -> u8 {
        bus.set_mreq(false);
        bus.set_rd(false);
        self.check_wait(bus);
        bus.set_m1(false);

        let addr = self.reg_pair(self.reg.spr.i, self.reg.spr.r);
        bus.set_addr(addr);
        bus.set_rfsh(true);

        bus.data()
    }

    pub(crate) fn fetch_t4(&mut self, _bus: &mut B) {
        // Only the lower 7 bits of R are incremented, 8th bit should remain unchanged
        let r = self.reg.spr.r;
        self.reg.spr.r = (r & (1 << 7)) | ((r + 1) & !(1 << 7));
    }

    pub(crate) fn mem_rd_t1(&mut self, bus: &mut B, addr: u16) {
        bus.set_addr(addr);
    }

    pub(crate) fn mem_rd_t2(&mut self, bus: &mut B) {
        bus.set_mreq(true);
        bus.set_rd(true);
    }

    pub(crate) fn mem_rd_t3(&mut self, bus: &mut B) -> u8 {
        bus.set_mreq(false);
        bus.set_rd(false);
        self.check_wait(bus);
        bus.data()
    }

    pub(crate) fn mem_wr_t1(&mut self, bus: &mut B, addr: u16) {
        bus.set_addr(addr);
    }

    pub(crate) fn mem_wr_t2(&mut self, bus: &mut B, data: u8) {
        bus.set_data(data);
        bus.set_mreq(true);
        bus.set_wr(true);
    }

    pub(crate) fn mem_wr_t3(&mut self, bus: &mut B) {
        bus.set_mreq(false);
        bus.set_wr(false);
        self.check_wait(bus);
    }

    pub(crate) fn io_rd_t1(&mut self, bus: &mut B, port: u8) {
        let addr = u16::from_be_bytes([self.reg.gpr.a, port]);
        bus.set_addr(addr);
    }

    pub(crate) fn io_rd_t2(&mut self, _bus: &mut B) {}

    pub(crate) fn io_rd_t3(&mut self, bus: &mut B) {
        bus.set_iorq(true);
        bus.set_rd(true);
    }

    pub(crate) fn io_rd_t4(&mut self, bus: &mut B) -> u8 {
        bus.set_iorq(false);
        bus.set_rd(false);
        self.check_wait(bus);
        bus.data()
    }

    pub(crate) fn io_wr_t1(&mut self, bus: &mut B, port: u8) {
        let addr = u16::from_be_bytes([self.reg.gpr.a, port]);
        bus.set_addr(addr);
    }

    pub(crate) fn io_wr_t2(&mut self, _bus: &mut B) {}

    pub(crate) fn io_wr_t3(&mut self, bus: &mut B, data: u8) {
        bus.set_iorq(true);
        bus.set_wr(true);
        bus.set_data(data);
    }

    pub(crate) fn io_wr_t4(&mut self, bus: &mut B) {
        bus.set_iorq(false);
        bus.set_wr(false);
        self.check_wait(bus);
    }

    pub(crate) fn end_instruction(&mut self, _bus: &mut B) {
        self.tcycle = 0;
    }
}
