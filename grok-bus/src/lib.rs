pub trait BusHandler {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, val: u8);
    fn port_read(&mut self, port: u8) -> u8;
    fn port_write(&mut self, port: u8, val: u8);

    // Peek memory without causing any side-effects that a read might have
    fn mem_peek(&mut self, addr: u16) -> u8 {
        self.mem_read(addr)
    }
}
