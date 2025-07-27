#![allow(dead_code)]

use grok_80::{Cpu, Gpr, Interrupts, Registers, Spr, Wpr};
use grok_bus::{BusHandlerZ80, BusZ80};
use serde::Deserialize;
use std::path::PathBuf;

struct Ram {
    data: [u8; 0x10000],
}

impl Ram {
    fn new() -> Self {
        Self {
            data: [0x00; 0x10000],
        }
    }

    fn tick(&mut self, bus: &mut BusZ80) {
        let addr = bus.addr() as usize;

        if bus.mreq() && bus.rd() {
            bus.set_data(self.data[addr]);
        } else if bus.mreq() && bus.wr() {
            self.data[addr] = bus.data();
        }
    }
}

struct Ports {
    data: [u8; 0x100],
}

impl Ports {
    fn new() -> Self {
        Self {
            data: [0x00; 0x100],
        }
    }

    fn tick(&mut self, bus: &mut BusZ80) {
        let addr = (bus.addr() as u8) as usize;

        if bus.iorq() && bus.rd() {
            bus.set_data(self.data[addr]);
        } else if bus.iorq() && bus.wr() {
            self.data[addr] = bus.data();
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
struct TestCpuState {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    i: u8,
    r: u8,
    ei: u8,
    wz: u16,
    ix: u16,
    iy: u16,
    af_: u16,
    bc_: u16,
    de_: u16,
    hl_: u16,
    im: u8,
    p: u8,
    q: u8,
    iff1: u8,
    iff2: u8,
}

impl From<&Cpu<BusZ80>> for TestCpuState {
    fn from(cpu: &Cpu<BusZ80>) -> Self {
        let reg = cpu.reg();
        let wz = u16::from_be_bytes([reg.wpr.w, reg.wpr.z]);
        let af_ = u16::from_be_bytes([reg.gpr_alt.a, reg.gpr_alt.f]);
        let bc_ = u16::from_be_bytes([reg.gpr_alt.b, reg.gpr_alt.c]);
        let de_ = u16::from_be_bytes([reg.gpr_alt.d, reg.gpr_alt.e]);
        let hl_ = u16::from_be_bytes([reg.gpr_alt.h, reg.gpr_alt.l]);
        let int = cpu.int();

        Self {
            pc: reg.spr.pc,
            sp: reg.spr.sp,
            a: reg.gpr.a,
            b: reg.gpr.b,
            c: reg.gpr.c,
            d: reg.gpr.d,
            e: reg.gpr.e,
            f: reg.gpr.f,
            h: reg.gpr.h,
            l: reg.gpr.l,
            i: reg.spr.i,
            r: reg.spr.r,
            ei: int.ei as u8,
            wz,
            ix: reg.spr.ix,
            iy: reg.spr.iy,
            af_,
            bc_,
            de_,
            hl_,
            im: int.im,
            p: 0, // TODO
            q: 0, // TODO
            iff1: int.iff1 as u8,
            iff2: int.iff2 as u8,
        }
    }
}

impl From<&TestCpuState> for Cpu<BusZ80> {
    fn from(cpu_state: &TestCpuState) -> Self {
        let gpr = Gpr {
            a: cpu_state.a,
            f: cpu_state.f,
            b: cpu_state.b,
            c: cpu_state.c,
            d: cpu_state.d,
            e: cpu_state.e,
            h: cpu_state.h,
            l: cpu_state.l,
        };

        let [a_, f_] = cpu_state.af_.to_be_bytes();
        let [b_, c_] = cpu_state.bc_.to_be_bytes();
        let [d_, e_] = cpu_state.de_.to_be_bytes();
        let [h_, l_] = cpu_state.hl_.to_be_bytes();
        let gpr_alt = Gpr {
            a: a_,
            f: f_,
            b: b_,
            c: c_,
            d: d_,
            e: e_,
            h: h_,
            l: l_,
        };

        let spr = Spr {
            pc: cpu_state.pc,
            sp: cpu_state.sp,
            ix: cpu_state.ix,
            iy: cpu_state.iy,
            i: cpu_state.i,
            r: cpu_state.r,
        };

        let [w, z] = cpu_state.wz.to_be_bytes();
        let wpr = Wpr { w, z };

        let reg = Registers {
            spr,
            gpr,
            gpr_alt,
            wpr,
            ir: 0,
        };

        let int = Interrupts {
            ei: cpu_state.ei == 1,
            iff1: cpu_state.iff1 == 1,
            iff2: cpu_state.iff2 == 1,
            im: cpu_state.im,
        };

        let mut cpu: Cpu<BusZ80> = Cpu::new();
        cpu.set_reg(reg);
        cpu.set_int(int);
        cpu
    }
}

#[derive(Deserialize)]
struct TestRamState {
    addr: u16,
    data: u8,
}

impl From<&Vec<TestRamState>> for Ram {
    fn from(ram_state: &Vec<TestRamState>) -> Self {
        let mut mem = Ram::new();

        for r in ram_state {
            mem.data[r.addr as usize] = r.data;
        }

        mem
    }
}

#[derive(Deserialize)]
struct TestPortState {
    addr: u16,
    data: u8,
    mode: char,
}

impl From<&Vec<TestPortState>> for Ports {
    fn from(port_state: &Vec<TestPortState>) -> Self {
        let mut ports = Ports::new();

        for p in port_state.iter().filter(|p| p.mode == 'r') {
            ports.data[(p.addr as u8) as usize] = p.data;
        }

        ports
    }
}

#[derive(Deserialize)]
struct TestBusState {
    addr: Option<u16>,
    data: Option<u8>,
    pins: String,
}

#[derive(Deserialize)]
struct TestState {
    #[serde(flatten)]
    cpu: TestCpuState,
    ram: Vec<TestRamState>,
}

#[derive(Deserialize)]
struct Test {
    name: String,
    #[serde(rename = "initial")]
    initial_state: TestState,
    #[serde(rename = "final")]
    final_state: TestState,
    cycles: Vec<TestBusState>,
    ports: Option<Vec<TestPortState>>,
}

fn bus_to_str(bus: &BusZ80) -> String {
    let r = if bus.rd() { 'r' } else { '-' };
    let w = if bus.wr() { 'w' } else { '-' };
    let m = if bus.mreq() { 'm' } else { '-' };
    let i = if bus.iorq() { 'i' } else { '-' };
    [r, w, m, i].iter().collect()
}

fn parse_test(path: &PathBuf) -> Vec<Test> {
    let data = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn opcode_test(path: &PathBuf) {
    let tests = parse_test(path);
    for t in &tests {
        if t.name == "D3 00BC" {
            continue;
        }
        let mut bus = BusZ80::new();
        let mut cpu = Cpu::from(&t.initial_state.cpu);
        let mut ram = Ram::from(&t.initial_state.ram);
        let mut ports = t.ports.as_ref().map(Ports::from);

        // Execute each t-cycle of the instruction
        for (i, b) in t.cycles.iter().enumerate() {
            // Test bus pins state
            let actual = bus_to_str(&bus);
            let expected = &b.pins;
            assert!(
                actual == *expected,
                "\nTest {}, Pins, Cycle: {}/{}:\nActual: {}\nExpected: {}\n",
                t.name,
                i + 1,
                t.cycles.len(),
                actual,
                expected,
            );

            // Tick components
            cpu.tick(&mut bus);
            ram.tick(&mut bus);
            if let Some(ports) = &mut ports {
                ports.tick(&mut bus);
            }

            // Test bus addr state
            if let Some(expected) = b.addr {
                let actual = bus.addr();
                assert!(
                    actual == expected,
                    "\nTest {}, Addr, Cycle: {}/{}:\nActual: {}\nExpected: {}\n",
                    t.name,
                    i + 1,
                    t.cycles.len(),
                    actual,
                    expected,
                );
            }

            // Test bus data state
            if let Some(expected) = b.data {
                let actual = bus.data();
                assert!(
                    actual == expected,
                    "\nTest {}, Data, Cycle: {}/{}:\nActual: {}\nExpected: {}\n",
                    t.name,
                    i + 1,
                    t.cycles.len(),
                    actual,
                    expected,
                );
            }
        }

        // Test final CPU state
        let initial = &t.initial_state.cpu;
        let actual = TestCpuState::from(&cpu);
        let expected = &t.final_state.cpu;
        assert!(
            actual == *expected,
            "\nTest {}, CPU State:\nInitial:\n{:?}\n\nFinal Actual:\n{:?}\n\nFinal Expected:\n{:?}\n",
            t.name,
            initial,
            actual,
            expected,
        );

        // Test final RAM state
        for r in &t.final_state.ram {
            let actual = ram.data[r.addr as usize];
            let expected = r.data;
            assert!(
                actual == expected,
                "\nTest {}, RAM State:\nActual: {}\nExpected: {}\n",
                t.name,
                actual,
                expected,
            );
        }
    }
}

#[test]
fn cpu_test_41() {
    opcode_test(&PathBuf::from("tests/opcodes/41.json"));
}

#[test]
fn cpu_test_46() {
    opcode_test(&PathBuf::from("tests/opcodes/46.json"));
}

#[test]
fn cpu_test_70() {
    opcode_test(&PathBuf::from("tests/opcodes/70.json"));
}

#[test]
fn cpu_test_db() {
    opcode_test(&PathBuf::from("tests/opcodes/db.json"));
}

#[test]
fn cpu_test_d3() {
    opcode_test(&PathBuf::from("tests/opcodes/d3.json"));
}
