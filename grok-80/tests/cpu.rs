use grok_80::{Cpu, Interrupts, Registers};
use grok_bus::{BusHandlerZ80, BusZ80};
use rstest::*;
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
struct TestCpu {
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

impl From<&Cpu<BusZ80>> for TestCpu {
    fn from(cpu: &Cpu<BusZ80>) -> Self {
        let reg = cpu.reg();
        let af_ = u16::from_be_bytes([reg.a_, reg.f_]);
        let bc_ = u16::from_be_bytes([reg.b_, reg.c_]);
        let de_ = u16::from_be_bytes([reg.d_, reg.e_]);
        let hl_ = u16::from_be_bytes([reg.h_, reg.l_]);
        let int = cpu.int();

        Self {
            pc: reg.pc,
            sp: reg.sp,
            a: reg.a,
            b: reg.b,
            c: reg.c,
            d: reg.d,
            e: reg.e,
            f: reg.f,
            h: reg.h,
            l: reg.l,
            i: reg.i,
            r: reg.r,
            ei: int.ei as u8,
            wz: reg.wz,
            ix: reg.ix,
            iy: reg.iy,
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

impl From<&TestCpu> for Cpu<BusZ80> {
    fn from(test_cpu: &TestCpu) -> Self {
        let [a_, f_] = test_cpu.af_.to_be_bytes();
        let [b_, c_] = test_cpu.bc_.to_be_bytes();
        let [d_, e_] = test_cpu.de_.to_be_bytes();
        let [h_, l_] = test_cpu.hl_.to_be_bytes();

        let reg = Registers {
            pc: test_cpu.pc,
            sp: test_cpu.sp,

            a: test_cpu.a,
            a_,

            f: test_cpu.f,
            f_,

            b: test_cpu.b,
            c: test_cpu.c,
            d: test_cpu.d,
            e: test_cpu.e,
            h: test_cpu.h,
            l: test_cpu.l,
            b_,
            c_,
            d_,
            e_,
            h_,
            l_,

            ix: test_cpu.ix,
            iy: test_cpu.iy,

            i: test_cpu.i,
            r: test_cpu.r,

            wz: test_cpu.wz,

            ir: 0,
            ir_pre: 0,

            tmp: [0x00; 2],
        };

        let int = Interrupts {
            ei: test_cpu.ei == 1,
            iff1: test_cpu.iff1 == 1,
            iff2: test_cpu.iff2 == 1,
            im: test_cpu.im,
        };

        let mut cpu: Cpu<BusZ80> = Cpu::new();
        cpu.set_reg(reg);
        cpu.set_int(int);
        cpu
    }
}

#[derive(Deserialize)]
struct TestRam {
    addr: u16,
    data: u8,
}

impl From<&Vec<TestRam>> for Ram {
    fn from(test_ram: &Vec<TestRam>) -> Self {
        let mut mem = Ram::new();

        for r in test_ram {
            mem.data[r.addr as usize] = r.data;
        }

        mem
    }
}

#[derive(Deserialize)]
struct TestPorts {
    addr: u16,
    data: u8,
    mode: char,
}

impl From<&Vec<TestPorts>> for Ports {
    fn from(test_ports: &Vec<TestPorts>) -> Self {
        let mut ports = Ports::new();

        for p in test_ports.iter().filter(|p| p.mode == 'r') {
            ports.data[(p.addr as u8) as usize] = p.data;
        }

        ports
    }
}

#[derive(Deserialize)]
struct TestBus {
    addr: Option<u16>,
    data: Option<u8>,
    signals: String,
}

#[derive(Deserialize)]
struct TestSystem {
    #[serde(flatten)]
    cpu: TestCpu,
    ram: Vec<TestRam>,
}

#[derive(Deserialize)]
struct Test {
    name: String,
    #[serde(rename = "initial")]
    initial_state: TestSystem,
    #[serde(rename = "final")]
    final_state: TestSystem,
    cycles: Vec<TestBus>,
    ports: Option<Vec<TestPorts>>,
}

fn parse_test(path: &PathBuf) -> Vec<Test> {
    let data = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn test_bus_state(bus: &BusZ80, test_bus: &TestBus, test: &Test, cycle: usize) {
    // Test bus signals state
    let actual: String = {
        let r = if bus.rd() { 'r' } else { '-' };
        let w = if bus.wr() { 'w' } else { '-' };
        let m = if bus.mreq() { 'm' } else { '-' };
        let i = if bus.iorq() { 'i' } else { '-' };
        [r, w, m, i].iter().collect()
    };
    let expected = &test_bus.signals;
    assert!(
        actual == *expected,
        "\nTest {}, Bus Signals, Cycle: {}/{}:\nActual: {}\nExpected: {}\n",
        test.name,
        cycle + 1,
        test.cycles.len(),
        actual,
        expected,
    );

    // Test addr bus state
    if let Some(expected) = test_bus.addr {
        let actual = bus.addr();
        assert!(
            actual == expected,
            "\nTest {}, Addr Bus, Cycle: {}/{}:\nActual: {}\nExpected: {}\n",
            test.name,
            cycle + 1,
            test.cycles.len(),
            actual,
            expected,
        );
    }

    // Test data bus state
    if let Some(expected) = test_bus.data {
        let actual = bus.data();
        assert!(
            actual == expected,
            "\nTest {}, Data Bus, Cycle: {}/{}:\nActual: {}\nExpected: {}\n",
            test.name,
            cycle + 1,
            test.cycles.len(),
            actual,
            expected,
        );
    }
}

fn test_cpu_state(cpu: &Cpu<BusZ80>, test: &Test) {
    let initial = &test.initial_state.cpu;
    let actual = TestCpu::from(cpu);
    let expected = &test.final_state.cpu;
    assert!(
        actual == *expected,
        "\nTest {}, CPU:\nInitial:\n{:?}\n\n Actual:\n{:?}\n\nExpected:\n{:?}\n",
        test.name,
        initial,
        actual,
        expected,
    );
}

fn test_ram_state(ram: &Ram, test: &Test) {
    for test_ram in &test.final_state.ram {
        let actual = ram.data[test_ram.addr as usize];
        let expected = test_ram.data;
        assert!(
            actual == expected,
            "\nTest {}, RAM @ {}:\nActual: {}\nExpected: {}\n",
            test.name,
            test_ram.addr,
            actual,
            expected,
        );
    }
}

fn test_ports_state(ports: &Ports, test: &Test) {
    let test_ports_list = test
        .ports
        .as_ref()
        .expect("Should not be called if there are no test ports");

    for test_ports in test_ports_list {
        let actual = ports.data[(test_ports.addr as u8) as usize];
        let expected = test_ports.data;
        assert!(
            actual == expected,
            "\nTest {}, Port @ {}:\nActual: {}\nExpected: {}\n",
            test.name,
            test_ports.addr,
            actual,
            expected,
        );
    }
}

fn test_instruction(path: &PathBuf) {
    let tests = parse_test(path);
    for test in &tests {
        let mut bus = BusZ80::new();
        let mut cpu = Cpu::from(&test.initial_state.cpu);
        let mut ram = Ram::from(&test.initial_state.ram);
        let mut ports = test.ports.as_ref().map(Ports::from);

        // Execute each t-cycle and test the bus state
        for (cycle, test_bus) in test.cycles.iter().enumerate() {
            cpu.tick(&mut bus);
            ram.tick(&mut bus);
            if let Some(ports) = &mut ports {
                ports.tick(&mut bus);
            }

            test_bus_state(&bus, test_bus, test, cycle);
        }

        // Then test final system state
        test_cpu_state(&cpu, test);
        test_ram_state(&ram, test);
        if let Some(ports) = &ports {
            test_ports_state(ports, test);
        }
    }
}

#[rstest]
fn cpu_test(#[files("tests/single-step-tests/*.json")] path: PathBuf) {
    test_instruction(&path);
}
