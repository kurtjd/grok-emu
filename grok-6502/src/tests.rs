use crate::*;
use bus::Bus;
use rstest::*;
use serde::Deserialize;
use std::path::PathBuf;

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

// Reset the CPU (which takes 14 clock phases to complete)
fn reset_cpu(cpu: &mut Cpu, bus: &mut dyn bus::Bus) {
    // Toggle reset pin to kick off the reset sequence
    bus.set_res(true);
    bus.tick();
    cpu.tick(bus);
    bus.set_res(false);

    // Then wait for reset sequence to complete
    for _ in 0..14 {
        bus.tick();
        cpu.tick(bus);
    }
}

fn opcode_test(path: &PathBuf) {
    let mut bus = bus::SimpleBus::new();
    let mut cpu = Cpu::new();
    let mut memory = Memory::default();

    let tests = parse_test(path);
    for t in &tests {
        // We overwrite the state below but this ensures we aren't in HALT state
        // (also just helps sanity check reset logic)
        reset_cpu(&mut cpu, &mut bus);

        // Set the initial state of the CPU
        t.initial_state.cpu.set_state(&mut cpu);

        // Set the initial state of RAM
        for r in &t.initial_state.ram {
            memory.ram[r.address as usize] = r.value;
        }

        // Execute each cycle of opcode
        for (cycle, expected) in t.cycles.iter().enumerate() {
            // 1st clock phase
            cpu.tick(&mut bus);
            memory.tick(&mut bus);
            bus.tick();
            test_bus_state(&bus, expected, t, cycle);

            // 2nd clock phase
            cpu.tick(&mut bus);
        }

        // Check the final state of the CPU
        test_cpu_state(&cpu, t);

        // Check the final state of RAM
        test_ram_state(&memory, t);

        // Ensure we are now starting the next opcode (aka we finished this one)
        if cpu.state() != State::Halt {
            cpu.tick(&mut bus);
            assert!(bus.sync());
        }
    }
}

#[rstest]
fn cpu_test(#[files("single-step-tests/*.json")] path: PathBuf) {
    opcode_test(&path);
}
