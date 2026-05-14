use crate::*;
use bus::Bus;
use rstest::*;
use serde::Deserialize;
use std::path::PathBuf;

const ILLEGAL_OPCODES: [u8; 105] = [
    0x1A, 0x3A, 0x5A, 0x7A, 0xDA, 0xFA, 0x80, 0x82, 0x89, 0xC2, 0xE2, 0x04, 0x44, 0x64, 0x14, 0x34,
    0x54, 0x74, 0xD4, 0xF4, 0x0C, 0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC, 0x4B, 0x0B, 0x2B, 0x8B, 0x6B,
    0xC7, 0xD7, 0xCF, 0xDF, 0xDB, 0xC3, 0xD3, 0xE7, 0xF7, 0xEF, 0xFF, 0xFB, 0xE3, 0xF3, 0xBB, 0xA7,
    0xB7, 0xAF, 0xBF, 0xA3, 0xB3, 0xAB, 0x27, 0x37, 0x2F, 0x3F, 0x3B, 0x23, 0x33, 0x67, 0x77, 0x6F,
    0x7F, 0x7B, 0x63, 0x73, 0x87, 0x97, 0x8F, 0x83, 0xCB, 0x9F, 0x93, 0x9E, 0x9C, 0x07, 0x17, 0x0F,
    0x1F, 0x1B, 0x03, 0x13, 0x47, 0x57, 0x4F, 0x5F, 0x5B, 0x43, 0x53, 0x9B, 0xEB, 0x02, 0x12, 0x22,
    0x32, 0x42, 0x52, 0x62, 0x72, 0x92, 0xB2, 0xD2, 0xF2,
];

const MEM_SIZE: usize = 0x10000;

#[derive(Deserialize)]
struct TestRam {
    address: u16,
    value: u8,
}

#[derive(Deserialize)]
struct TestBus {
    address: u16,
    value: u8,
    ctype: String,
}

#[derive(Deserialize, PartialEq, Debug)]
struct TestCpu {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
}

#[derive(Deserialize)]
struct TestState {
    #[serde(flatten)]
    cpu: TestCpu,
    ram: Vec<TestRam>,
}

#[derive(Deserialize)]
struct Test {
    name: String,
    #[serde(rename = "initial")]
    initial_state: TestState,
    #[serde(rename = "final")]
    final_state: TestState,
    cycles: Vec<TestBus>,
}

impl From<&TestCpu> for TestCpu {
    fn from(tc: &TestCpu) -> Self {
        Self {
            pc: tc.pc,
            s: tc.s,
            a: tc.a,
            x: tc.x,
            y: tc.y,
            p: tc.p,
        }
    }
}

impl TestCpu {
    fn set_state(&self, cpu: &mut Cpu) {
        cpu.registers.pc = self.pc;
        cpu.registers.s = self.s;
        cpu.registers.a = self.a;
        cpu.registers.x = self.x;
        cpu.registers.y = self.y;
        cpu.registers.p = StatusFlags::from_bits_retain(self.p);
    }

    fn state(cpu: &Cpu) -> Self {
        Self {
            pc: cpu.registers.pc,
            s: cpu.registers.s,
            a: cpu.registers.a,
            x: cpu.registers.x,
            y: cpu.registers.y,
            p: cpu.registers.p.bits(),
        }
    }
}

struct Memory {
    ram: [u8; MEM_SIZE],
}

impl Memory {
    fn tick(&mut self, bus: &mut dyn bus::Bus) {
        match bus.op() {
            bus::Op::Read => bus.set_data(self.ram[bus.addr() as usize]),
            bus::Op::Write => self.ram[bus.addr() as usize] = bus.data(),
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Memory { ram: [0; MEM_SIZE] }
    }
}

impl From<&Vec<TestRam>> for Memory {
    fn from(test_ram: &Vec<TestRam>) -> Self {
        let mut mem = Memory::default();
        for r in test_ram {
            mem.ram[r.address as usize] = r.value;
        }
        mem
    }
}

fn parse_test(path: &PathBuf) -> Vec<Test> {
    let data = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn test_bus_state(bus: &dyn bus::Bus, expected: &TestBus, test: &Test, cycle: usize) {
    assert_eq!(
        bus.addr(),
        expected.address,
        "\nTest {}, Addr Bus, Cycle {}/{}:\nActual: {}\nExpected: {}\n",
        test.name,
        cycle + 1,
        test.cycles.len(),
        bus.addr(),
        expected.address,
    );
    assert_eq!(
        bus.data(),
        expected.value,
        "\nTest {}, Data Bus, Cycle {}/{}:\nActual: {}\nExpected: {}\n",
        test.name,
        cycle + 1,
        test.cycles.len(),
        bus.data(),
        expected.value,
    );

    let actual_op = match bus.op() {
        bus::Op::Read => "read",
        bus::Op::Write => "write",
    };
    assert_eq!(
        actual_op,
        expected.ctype,
        "\nTest {}, Bus Op, Cycle {}/{}:\nActual: {}\nExpected: {}\n",
        test.name,
        cycle + 1,
        test.cycles.len(),
        actual_op,
        expected.ctype,
    );
}

fn test_cpu_state(cpu: &Cpu, test: &Test) {
    let initial = &test.initial_state.cpu;
    let actual = TestCpu::state(cpu);
    let expected = &test.final_state.cpu;
    assert!(
        actual == *expected,
        "\nTest {}, CPU:\nInitial:\n{:?}\n\nActual:\n{:?}\n\nExpected:\n{:?}\n",
        test.name,
        initial,
        actual,
        expected,
    );
}

fn test_ram_state(memory: &Memory, test: &Test) {
    for m in &test.final_state.ram {
        assert_eq!(
            memory.ram[m.address as usize], m.value,
            "\nTest {}, RAM @ {}:\nActual: {}\nExpected: {}\n",
            test.name, m.address, memory.ram[m.address as usize], m.value,
        );
    }
}

fn opcode_test(path: &PathBuf) {
    let mut bus = bus::SimpleBus::new();
    let mut memory = Memory::default();

    // CPU starts in reset state which takes 7 cycles to complete
    let mut cpu = Cpu::new();
    for _ in 0..14 {
        cpu.tick(&mut bus);
        memory.tick(&mut bus);
    }

    let tests = parse_test(path);
    for t in &tests {
        // Don't have cycle accuracy for illegal opcodes yet, so don't test
        let opcode = u8::from_str_radix(&t.name[0..2], 16).unwrap();
        if ILLEGAL_OPCODES.contains(&opcode) {
            continue;
        }

        // Set the initial state of the CPU
        t.initial_state.cpu.set_state(&mut cpu);

        // Set the initial state of RAM
        for r in &t.initial_state.ram {
            memory.ram[r.address as usize] = r.value;
        }

        // Execute each cycle of opcode
        for (cycle, expected) in t.cycles.iter().enumerate() {
            // First half cycle
            cpu.tick(&mut bus);
            memory.tick(&mut bus);
            test_bus_state(&bus, expected, t, cycle);

            // 2nd half cycle
            cpu.tick(&mut bus);
        }

        // Ensure we are now starting the next opcode (aka we finished this one)
        assert!(bus.sync());

        // Check the final state of the CPU
        test_cpu_state(&cpu, t);

        // Check the final state of RAM
        test_ram_state(&memory, t);
    }
}

#[rstest]
fn cpu_test(#[files("single-step-tests/*.json")] path: PathBuf) {
    opcode_test(&path);
}
