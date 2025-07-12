use grok_80::{BusHandler, Cpu};
use grok_dbg::Debugger;
use std::io::{Read, Write};

fn read_rom_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

struct Bus {
    ram: [u8; u16::MAX as usize + 1],
    use_cpm: bool,
}

impl Bus {
    fn new(use_cpm: bool) -> Self {
        Self {
            ram: [0; u16::MAX as usize + 1],
            use_cpm,
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

    fn port_read(&mut self, port: u8) -> u8 {
        println!("IN port={port}");
        0
    }

    fn port_write(&mut self, port: u8, val: u8) {
        // We treat port 0 as a putc call
        if self.use_cpm && port == 0 {
            print!("{}", val as char);
            std::io::stdout().flush().unwrap();
        } else {
            println!("OUT port={port} val=0x{val:02X}");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: cargo run <ROM path> <load addr> [--cpm]")
    }

    let rom_path = args[1].as_str();
    let load_addr = {
        let addr = args[2].trim_start_matches("0x");
        u16::from_str_radix(addr, 16).expect("Load addr must be in 16-bit hex format")
    };

    let use_cpm = args.len() == 4 && args[3] == "--cpm";
    let rom = read_rom_bytes(rom_path).expect("Error loading ROM: {rom_path}");

    let mut bus = Bus::new(use_cpm);
    bus.mem_load(load_addr, &rom);

    // Load fake CP/M stub
    if use_cpm {
        bus.mem_load(0x00, include_bytes!("../roms/CPM.bin"));
    }

    let mut cpu = Cpu::new(bus);
    cpu.reset(load_addr);
    let mut dbg = Debugger::new(cpu);

    dbg.start();
}
