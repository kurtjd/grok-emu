use grok_80::{Cpu, Opcode};
use grok_bus::BusHandler;

struct Bus {
    ram: [u8; 0x10000],
    output: String,
    exit: bool,
}

impl Bus {
    fn new() -> Self {
        Self {
            ram: [0; 0x10000],
            output: String::new(),
            exit: false,
        }
    }

    fn mem_load(&mut self, addr: u16, bytes: &[u8]) {
        for (i, val) in bytes.iter().enumerate() {
            self.ram[addr as usize + i] = *val;
        }
    }
}

impl BusHandler for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }

    fn port_read(&mut self, _port: u8) -> u8 {
        panic!("IN not supported");
    }

    fn port_write(&mut self, port: u8, val: u8) {
        // We treat port 0 as a putc call
        if port == 0 {
            self.output.push(val as char);
            print!("{}", val as char);

        // And port 1 as an exit call
        } else if port == 1 {
            self.exit = true;
        } else {
            panic!("Unsupported OUT port");
        }
    }
}

fn run_test(rom: &[u8]) -> bool {
    let mut bus = Bus::new();

    // Load fake CP/M stub
    bus.mem_load(0x00, include_bytes!("../roms/CPM.bin"));

    // Tests jump to 0x00 when complete, so insert an OUT here for exit
    bus.mem_load(0x00, &[Opcode::OUT as u8, 0x01]);

    // Test ROMs expect to be loaded at 0x100
    bus.mem_load(0x100, rom);

    let mut cpu: Cpu<Bus> = Cpu::new();
    cpu.reset(0x100);

    while !bus.exit {
        cpu.step(&mut bus);
    }

    !bus.output.contains("FAILED") && !bus.output.contains("ERROR")
}

#[test]
fn test_tst8080() {
    assert!(run_test(include_bytes!("../roms/TST8080.COM")));
}

#[test]
fn test_cputest() {
    assert!(run_test(include_bytes!("../roms/CPUTEST.COM")));
}

#[test]
fn test_8080pre() {
    assert!(run_test(include_bytes!("../roms/8080PRE.COM")));
}

#[test]
fn test_8080exm() {
    assert!(run_test(include_bytes!("../roms/8080EXM.COM")));
}
