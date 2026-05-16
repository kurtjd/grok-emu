//! MOS 6502 emulator.

pub mod bus;
mod opcodes;
#[cfg(test)]
mod tests;

use bitflags::bitflags;
use bus::Bus;

const STACK_OFFSET: u16 = 0x0100;
const RESET_VECTOR: u16 = 0xFFFC;
const INTR_VECTOR: u16 = 0xFFFE;

#[derive(Default)]
enum State {
    Reset,
    #[default]
    Run,
    Halt,
}

bitflags! {
    #[derive(Default, Clone, Copy)]
    struct StatusFlags: u8 {
        const N = 1 << 7;   // Negative
        const V = 1 << 6;   // Overflow
        const E = 1 << 5;   // Extension (unused, but initialized to 1)
        const B = 1 << 4;   // Break
        const D = 1 << 3;   // Decimal
        const I = 1 << 2;   // Interrupt Disable
        const Z = 1 << 1;   // Zero
        const C = 1 << 0;   // Carry
    }
}

// The addressing mode of an instruction
//
// This is the main thing that determines cycles and bus activity of most instructions,
// so we use this as the first later of dispatch
enum AddrMode {
    Acm0, // Accumulator
    Abs0, // Absolute
    AbsX, // Absolute Indexed with X
    AbsY, // Absolute Indexed with Y
    Imm0, // Immediate
    Imp0, // Implied
    Ind0, // Indirect
    IndX, // Indirect Indexed with X
    IndY, // Indirect Indexed with Y
    Rel0, // Relative
    Zpg0, // Zero Page
    ZpgX, // Zero Page Indexed Indirect with X
    ZpgY, // Zero Page Indexed Indirect with Y
}

// Represents the "type" of instruction, mostly matching the groupings from the hardware manual
//
// `Misc` represents instructions that have unique bus behavior and need manual control
// over their cycles
//
// This is basically the 2nd layer of dispatch (after address mode dispatching)
#[derive(Clone, Copy)]
pub(crate) enum Instruction {
    SingleByte(fn(&mut Cpu)),
    Read(fn(&mut Cpu, data: u8)),
    Write(fn(&mut Cpu) -> u8),
    Rmw(fn(&mut Cpu, data: u8) -> u8),
    Branch(fn(&mut Cpu) -> bool),
    Push(fn(&mut Cpu) -> u8),
    Pull(fn(&mut Cpu, data: u8)),
    Jmp(fn(&mut Cpu, addr: u16)),
    // Annoying edge case because this guy can overwrite the address it is writing to
    Shr(fn(&mut Cpu) -> u8),
    Misc(fn(&mut Cpu, &mut dyn Bus)),
}

// Stuff just used by the emulator internally, but not used by instructions directly
struct Internal {
    // Scratch mainly for holding intermediate values between ticks during instruction execution
    scratch: [u8; 2],
    scratch2: [u8; 2],

    // Instruction register to hold the current opcode being executed
    ir: &'static opcodes::Opcode,
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            scratch: [0; 2],
            scratch2: [0; 2],
            ir: &opcodes::OPCODES[0],
        }
    }
}

// All registers of the 6502
#[derive(Default)]
struct Registers {
    pc: u16,            // Program counter
    s: u8,              // Stack pointer
    a: u8,              // Accumulator
    x: u8,              // X register
    y: u8,              // Y register
    p: StatusFlags,     // Status
    internal: Internal, // Private registers not used by instructions
}

/// A clock-phase stepped MOS 6502 emulator.
#[derive(Default)]
pub struct Cpu {
    state: State,
    registers: Registers,
    hcycle: u8,
}

impl Cpu {
    /// Create a new instance of the CPU in the `Run` state.
    ///
    /// The reset sequence should be performed to initialize CPU.
    pub fn new() -> Self {
        Self::default()
    }

    /// Put the CPU into reset state, which will cause it
    /// to begin the 7 cycle (14 tick) reset sequence on next tick.
    ///
    /// This is also called automatically if the RES pin on the bus transitions from
    /// the active state to inactive state.
    pub fn reset(&mut self) {
        self.state = State::Reset;
        self.hcycle = 0;

        // Disable interrupts flag and extension bit should be set
        self.registers.p = StatusFlags::E | StatusFlags::I;
    }

    /// Put the CPU into halt state, which will cause it to do nothing every tick.
    ///
    /// The only way to recover from the halt state is to reset the CPU
    /// (either from a reset signal on the bus or by calling `reset()` directly).
    pub fn halt(&mut self) {
        self.state = State::Halt;
    }

    /// Advance the CPU state by one clock phase (half-cycle).
    ///
    /// Bus activity can be observed after the first clock phase by other hardware
    /// (such as memory placing data on the bus) which the CPU can then react to
    /// in the second clock phase.
    pub fn tick(&mut self, bus: &mut dyn Bus) {
        // When RST goes active, the CPU enters a halt state
        // Only when it goes back inactive does the reset sequence actually begin
        match bus.res_edge() {
            Some(true) => self.halt(),
            Some(false) => self.reset(),
            None => (),
        }

        // Sync should be active for first cycle,
        // but this gets observed AFTER the first cycle completes,
        // hence why we set it inactive (end_instruction sets it active)
        bus.set_sync(false);

        // An active edge of SO tells us we should set the overflow flag
        if bus.so_edge() == Some(true) {
            self.registers.p.insert(StatusFlags::V);
        }

        // If RDY is inactive and we are on the second clock phase of a read cycle,
        // we need to pause until it goes active again
        if !self.hcycle.is_multiple_of(2) && !bus.rdy() && bus.op() == bus::Op::Read {
            return;
        }

        self.hcycle += 1;
        match self.state {
            State::Reset => match self.hcycle {
                // T0 (Dummy fetch)
                1..=2 => self.fetch(bus),

                // T1 (Dummy read)
                3 => self.fetch_pc(bus),
                4 => (),

                // T2 (Fake push)
                5 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
                6 => self.registers.s = self.registers.s.wrapping_sub(1),

                // T3 (Fake push)
                7 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
                8 => self.registers.s = self.registers.s.wrapping_sub(1),

                // T4 (Fake push)
                9 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
                10 => self.registers.s = self.registers.s.wrapping_sub(1),

                // T5 (Fetch reset vector low byte)
                11 => bus.start_read(RESET_VECTOR),
                12 => self.registers.internal.scratch[0] = bus.data(),

                // T6 (Fetch reset vector high byte + jump)
                13 => bus.start_read(RESET_VECTOR + 1),
                14 => {
                    self.registers.internal.scratch[1] = bus.data();
                    self.registers.pc = u16::from_le_bytes(self.registers.internal.scratch);
                    self.state = State::Run;
                    self.end_instruction(bus);
                }

                _ => unreachable!(),
            },
            State::Run => match self.hcycle {
                // T0 (Fetch opcode)
                1..=2 => self.fetch(bus),

                // T1+
                3.. => self.dispatch(bus),
                _ => unreachable!(),
            },
            State::Halt => self.hcycle = 0,
        }
    }

    fn end_instruction(&mut self, bus: &mut dyn Bus) {
        self.hcycle = 0;

        // Sync pin goes active when next instruction starts (aka when this one ends)
        bus.set_sync(true);
    }

    fn stack_push(&mut self, bus: &mut dyn Bus, data: u8) {
        bus.start_write(STACK_OFFSET + self.registers.s as u16, data);
        self.registers.s = self.registers.s.wrapping_sub(1);
    }

    fn stack_pop(&mut self, bus: &mut dyn Bus) {
        self.registers.s = self.registers.s.wrapping_add(1);
        bus.start_read(STACK_OFFSET + self.registers.s as u16);
    }

    fn fetch_pc(&mut self, bus: &mut dyn Bus) {
        bus.start_read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
    }

    fn fetch(&mut self, bus: &mut dyn Bus) {
        match self.hcycle {
            // T0 (Fetch opcode)
            1 => self.fetch_pc(bus),
            2 => self.registers.internal.ir = &opcodes::OPCODES[bus.data() as usize],
            _ => unreachable!(),
        }
    }

    fn dispatch_acm0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;
        match self.hcycle {
            // T1 (Dummy read + execute)
            3 => bus.start_read(self.registers.pc),
            4 => match opcode.instr {
                Instruction::Rmw(exec) => {
                    self.registers.a = exec(self, self.registers.a);
                    self.end_instruction(bus);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn dispatch_abs0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch low byte of address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[0] = bus.data(),

            // T2 (Fetch high byte of address)
            5 => self.fetch_pc(bus),
            6 => {
                self.registers.internal.scratch[1] = bus.data();

                // JMP instructions always end here
                if let Instruction::Jmp(exec) = opcode.instr {
                    let addr = u16::from_le_bytes(self.registers.internal.scratch);
                    exec(self, addr);
                    self.end_instruction(bus);
                }
            }

            7.. => {
                let addr = u16::from_le_bytes(self.registers.internal.scratch);

                match opcode.instr {
                    Instruction::Read(exec) => match self.hcycle {
                        // T3 (Fetch data + execute)
                        7 => bus.start_read(addr),
                        8 => {
                            exec(self, bus.data());
                            self.end_instruction(bus);
                        }
                        _ => unreachable!(),
                    },
                    Instruction::Write(exec) => match self.hcycle {
                        // T3 (Execute + write data)
                        7 => {
                            let data = exec(self);
                            bus.start_write(addr, data);
                        }
                        8 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    Instruction::Rmw(exec) => match self.hcycle {
                        // T3 (Fetch data)
                        7 => bus.start_read(addr),
                        8 => self.registers.internal.scratch2[0] = bus.data(),

                        // T4 (Dummy write back unmodified data)
                        9 => bus.start_write(addr, self.registers.internal.scratch2[0]),
                        10 => (),

                        // T5 (Execute + write back modified data)
                        11 => {
                            let data = exec(self, self.registers.internal.scratch2[0]);
                            bus.start_write(addr, data);
                        }
                        12 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn dispatch_imp0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;
        match opcode.instr {
            Instruction::SingleByte(exec) => match self.hcycle {
                // T1 (Dummy read + execute)
                3 => bus.start_read(self.registers.pc),
                4 => {
                    exec(self);
                    self.end_instruction(bus);
                }
                _ => unreachable!(),
            },
            Instruction::Push(exec) => match self.hcycle {
                // T1 (Dummy read)
                3 => bus.start_read(self.registers.pc),
                4 => (),

                // T2 (Execute + push to stack)
                5 => {
                    let data = exec(self);
                    self.stack_push(bus, data);
                }
                6 => self.end_instruction(bus),
                _ => unreachable!(),
            },
            Instruction::Pull(exec) => match self.hcycle {
                // T1 (Dummy read)
                3 => bus.start_read(self.registers.pc),
                4 => (),

                // T2 (Dummy read)
                5 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
                6 => (),

                // T3 (Pop from stack + execute)
                7 => self.stack_pop(bus),
                8 => {
                    exec(self, bus.data());
                    self.end_instruction(bus);
                }
                _ => unreachable!(),
            },
            Instruction::Misc(exec) => exec(self, bus),
            _ => unreachable!(),
        }
    }

    fn dispatch_imm0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;
        match self.hcycle {
            // T1 (Fetch immediate operand + execute)
            3 => self.fetch_pc(bus),
            4 => match opcode.instr {
                Instruction::Read(exec) => {
                    exec(self, bus.data());
                    self.end_instruction(bus);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn dispatch_abs_offset(&mut self, bus: &mut dyn Bus, offset: u8) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch low byte of address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[0] = bus.data(),

            // T2 (Fetch high byte of address)
            5 => self.fetch_pc(bus),
            6 => self.registers.internal.scratch[1] = bus.data(),

            // T3 (Fetch data at base offset address)
            7 => {
                let addr = u16::from_le_bytes([
                    self.registers.internal.scratch[0].wrapping_add(offset),
                    self.registers.internal.scratch[1],
                ]);
                bus.start_read(addr);
            }
            8 => {
                // Only `Read` instructions can exit early here
                if let Instruction::Read(exec) = opcode.instr {
                    let addr = u16::from_le_bytes(self.registers.internal.scratch);
                    let eff_addr = addr.wrapping_add(offset as u16);

                    // No page was crossed, so end here
                    if (eff_addr & 0xFF00) == (addr & 0xFF00) {
                        exec(self, bus.data());
                        self.end_instruction(bus);
                    }
                }
            }

            9.. => {
                let base_addr = u16::from_le_bytes(self.registers.internal.scratch);
                let eff_addr = base_addr.wrapping_add(offset as u16);

                match opcode.instr {
                    Instruction::Read(exec) => match self.hcycle {
                        // T4 (Fetch data at effective address + execute)
                        9 => bus.start_read(eff_addr),
                        10 => {
                            exec(self, bus.data());
                            self.end_instruction(bus);
                        }
                        _ => unreachable!(),
                    },
                    Instruction::Write(exec) => match self.hcycle {
                        // T4 (Execute + write data to effective address)
                        9 => {
                            let data = exec(self);
                            bus.start_write(eff_addr, data);
                        }
                        10 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    Instruction::Shr(exec) => match self.hcycle {
                        // T4 (Execute + write data to (unstable) effective address)
                        9 => {
                            let data = exec(self);
                            let (target, data) = self.shr(base_addr, eff_addr, data);
                            bus.start_write(target, data);
                        }
                        10 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    Instruction::Rmw(exec) => match self.hcycle {
                        // T4 (Fetch data at effective address)
                        9 => bus.start_read(eff_addr),
                        10 => self.registers.internal.scratch2[0] = bus.data(),

                        // T5 (Dummy write back unmodified data)
                        11 => bus.start_write(eff_addr, self.registers.internal.scratch2[0]),
                        12 => (),

                        // T6 (Execute + write back modified data)
                        13 => {
                            let data = exec(self, self.registers.internal.scratch2[0]);
                            bus.start_write(eff_addr, data);
                        }
                        14 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn dispatch_ind0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch low byte of indirect address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[0] = bus.data(),

            // T2 (Fetch high byte of indirect address)
            5 => self.fetch_pc(bus),
            6 => self.registers.internal.scratch[1] = bus.data(),

            // T3 (Fetch low byte of jump address)
            7 => {
                let addr = u16::from_le_bytes(self.registers.internal.scratch);
                bus.start_read(addr);
            }
            8 => self.registers.internal.scratch2[0] = bus.data(),

            // T4 (Fetch high byte of jump address + execute)
            9 => {
                let addr = u16::from_le_bytes([
                    self.registers.internal.scratch[0].wrapping_add(1),
                    self.registers.internal.scratch[1],
                ]);
                bus.start_read(addr);
            }
            10 => {
                self.registers.internal.scratch2[1] = bus.data();
                match opcode.instr {
                    Instruction::Jmp(exec) => {
                        let addr = u16::from_le_bytes(self.registers.internal.scratch2);
                        exec(self, addr);
                        self.end_instruction(bus);
                    }
                    Instruction::Misc(exec) => {
                        // Instruction will know address is in scratch2
                        exec(self, bus);
                        self.end_instruction(bus);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn dispatch_indx(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch pg0 base address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[1] = bus.data(),

            // T2 (Dummy read)
            5 => bus.start_read(self.registers.internal.scratch[1] as u16),
            6 => (),

            // T3 (Fetch low byte of effective address)
            7 => {
                self.registers.internal.scratch[1] =
                    self.registers.internal.scratch[1].wrapping_add(self.registers.x);
                bus.start_read(self.registers.internal.scratch[1] as u16);
            }
            8 => self.registers.internal.scratch[0] = bus.data(),

            // T4 (Fetch high byte of effective address)
            9 => {
                let addr = self.registers.internal.scratch[1].wrapping_add(1);
                bus.start_read(addr as u16);
            }
            10 => self.registers.internal.scratch[1] = bus.data(),

            11.. => {
                let eff_addr = u16::from_le_bytes(self.registers.internal.scratch);

                match opcode.instr {
                    Instruction::Read(exec) => match self.hcycle {
                        // T5 (Fetch data at effective address + execute)
                        11 => bus.start_read(eff_addr),
                        12 => {
                            exec(self, bus.data());
                            self.end_instruction(bus);
                        }
                        _ => unreachable!(),
                    },
                    Instruction::Write(exec) => match self.hcycle {
                        // T5 (Execute + write data to effective address)
                        11 => {
                            let data = exec(self);
                            bus.start_write(eff_addr, data);
                        }
                        12 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    // Note: This is only used by illegal opcodes
                    Instruction::Rmw(exec) => match self.hcycle {
                        // T5 (Fetch data at effective address)
                        11 => bus.start_read(eff_addr),
                        12 => self.registers.internal.scratch2[0] = bus.data(),

                        // T6 (Dummy write back unmodified data)
                        13 => bus.start_write(eff_addr, self.registers.internal.scratch2[0]),
                        14 => (),

                        // T7 (Execute + write back modified data)
                        15 => {
                            let data = exec(self, self.registers.internal.scratch2[0]);
                            bus.start_write(eff_addr, data);
                        }
                        16 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn dispatch_indy(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch pg0 indirect address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[1] = bus.data(),

            // T2 (Fetch low byte of base address)
            5 => bus.start_read(self.registers.internal.scratch[1] as u16),
            6 => self.registers.internal.scratch[0] = bus.data(),

            // T3 (Fetch high byte of base address)
            7 => {
                let addr = self.registers.internal.scratch[1].wrapping_add(1);
                bus.start_read(addr as u16);
            }
            8 => self.registers.internal.scratch[1] = bus.data(),

            // T4 (Read from offset base address)
            9 => {
                let addr = u16::from_le_bytes([
                    self.registers.internal.scratch[0].wrapping_add(self.registers.y),
                    self.registers.internal.scratch[1],
                ]);
                bus.start_read(addr);
            }
            10 => {
                // Only `Internal` instructions can exit early here
                // Otherwise this is just a dummy read and we need to continue regardless
                if let Instruction::Read(exec) = opcode.instr {
                    let addr = u16::from_le_bytes(self.registers.internal.scratch);
                    let eff_addr = addr.wrapping_add(self.registers.y as u16);

                    // No page was crossed, so end here
                    if (eff_addr & 0xFF00) == (addr & 0xFF00) {
                        exec(self, bus.data());
                        self.end_instruction(bus);
                    }
                }
            }

            11.. => {
                let addr = u16::from_le_bytes(self.registers.internal.scratch);
                let eff_addr = addr.wrapping_add(self.registers.y as u16);

                match opcode.instr {
                    Instruction::Read(exec) => match self.hcycle {
                        // T5 (Fetch data at effective address + execute)
                        11 => bus.start_read(eff_addr),
                        12 => {
                            exec(self, bus.data());
                            self.end_instruction(bus);
                        }
                        _ => unreachable!(),
                    },
                    Instruction::Write(exec) => match self.hcycle {
                        // T5 (Execute + write data to effective address)
                        11 => {
                            let data = exec(self);
                            bus.start_write(eff_addr, data);
                        }
                        12 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    Instruction::Shr(exec) => match self.hcycle {
                        // T5 (Execute + write data to (unstable) effective address)
                        11 => {
                            let data = exec(self);
                            let (target, data) = self.shr(addr, eff_addr, data);
                            bus.start_write(target, data);
                        }
                        12 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    // Note: This is only used by illegal opcodes
                    Instruction::Rmw(exec) => match self.hcycle {
                        // T5 (Fetch data at effective address)
                        11 => bus.start_read(eff_addr),
                        12 => self.registers.internal.scratch2[0] = bus.data(),

                        // T6 (Dummy write back unmodified data)
                        13 => bus.start_write(eff_addr, self.registers.internal.scratch2[0]),
                        14 => (),

                        // T7 (Execute + write back modified data)
                        15 => {
                            let data = exec(self, self.registers.internal.scratch2[0]);
                            bus.start_write(eff_addr, data);
                        }
                        16 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }

            _ => unreachable!(),
        }
    }

    fn dispatch_rel0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch branch offset + execute)
            3 => self.fetch_pc(bus),
            4 => {
                self.registers.internal.scratch[0] = bus.data();
                match opcode.instr {
                    Instruction::Branch(exec) => {
                        // If the branch isn't taken, we end early
                        if !exec(self) {
                            self.end_instruction(bus);
                        }
                    }

                    // If we got here and aren't a branch instruction that's a problem
                    // (only branch instructions should use REL0 mode)
                    _ => unreachable!(),
                }
            }

            // T2 (Dummy read at PC + branch (if no page cross))
            5 => bus.start_read(self.registers.pc),
            6 => {
                let eff_addr = (self.registers.pc as i32
                    + (self.registers.internal.scratch[0] as i8) as i32)
                    as u16;
                // Store eff addr for next cycle so don't need to recalculate it
                self.registers.internal.scratch = eff_addr.to_le_bytes();

                // If no page was crossed, we can end early and branch
                if (eff_addr & 0xFF00) == (self.registers.pc & 0xFF00) {
                    self.registers.pc = eff_addr;
                    self.end_instruction(bus);
                }
            }

            // T3 (Dummy read at PC + branch (if page cross))
            7 => {
                // We have to dummy read from the unfixed offset address
                let base_addr = u16::from_le_bytes([
                    self.registers.internal.scratch[0],
                    (self.registers.pc >> 8) as u8,
                ]);
                bus.start_read(base_addr);
            }
            8 => {
                // But actually jump to the fixed effective address
                self.registers.pc = u16::from_le_bytes(self.registers.internal.scratch);
                self.end_instruction(bus);
            }

            _ => unreachable!(),
        }
    }

    fn dispatch_zpg0(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch zero page address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[0] = bus.data(),

            5.. => {
                let addr = self.registers.internal.scratch[0] as u16;

                match opcode.instr {
                    Instruction::Read(exec) => match self.hcycle {
                        // T2 (Fetch data + execute)
                        5 => bus.start_read(addr),
                        6 => {
                            exec(self, bus.data());
                            self.end_instruction(bus);
                        }
                        _ => unreachable!(),
                    },
                    Instruction::Write(exec) => match self.hcycle {
                        // T2 (Execute + write data)
                        5 => {
                            let data = exec(self);
                            bus.start_write(addr, data);
                        }
                        6 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    Instruction::Rmw(exec) => match self.hcycle {
                        // T2 (Fetch data)
                        5 => bus.start_read(addr),
                        6 => self.registers.internal.scratch[1] = bus.data(),

                        // T3 (Dummy write back unmodified data)
                        7 => bus.start_write(addr, self.registers.internal.scratch[1]),
                        8 => (),

                        // T4 (Execute + write back modified data)
                        9 => {
                            let data = exec(self, self.registers.internal.scratch[1]);
                            bus.start_write(addr, data);
                        }
                        10 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn dispatch_zpg_offset(&mut self, bus: &mut dyn Bus, offset: u8) {
        let opcode = self.registers.internal.ir;

        match self.hcycle {
            // T1 (Fetch zero page address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[0] = bus.data(),

            // T2 (Dummy read)
            5 => bus.start_read(self.registers.internal.scratch[0] as u16),
            6 => (),

            7.. => {
                let eff_addr = self.registers.internal.scratch[0].wrapping_add(offset) as u16;

                match opcode.instr {
                    Instruction::Read(exec) => match self.hcycle {
                        // T3 (Fetch data at effective address + execute)
                        7 => bus.start_read(eff_addr),
                        8 => {
                            exec(self, bus.data());
                            self.end_instruction(bus);
                        }
                        _ => unreachable!(),
                    },
                    Instruction::Write(exec) => match self.hcycle {
                        // T3 (Execute + write data to effective address)
                        7 => {
                            let data = exec(self);
                            bus.start_write(eff_addr, data);
                        }
                        8 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    Instruction::Rmw(exec) => match self.hcycle {
                        // T3 (Fetch data at effective address)
                        7 => bus.start_read(eff_addr),
                        8 => self.registers.internal.scratch[1] = bus.data(),

                        // T4 (Dummy write back unmodified data)
                        9 => bus.start_write(eff_addr, self.registers.internal.scratch[1]),
                        10 => (),

                        // T5 (Execute + write back modified data)
                        11 => {
                            let data = exec(self, self.registers.internal.scratch[1]);
                            bus.start_write(eff_addr, data);
                        }
                        12 => self.end_instruction(bus),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn dispatch(&mut self, bus: &mut dyn Bus) {
        let opcode = self.registers.internal.ir;

        match opcode.mode {
            AddrMode::Acm0 => self.dispatch_acm0(bus),
            AddrMode::Abs0 => self.dispatch_abs0(bus),
            AddrMode::AbsX => self.dispatch_abs_offset(bus, self.registers.x),
            AddrMode::AbsY => self.dispatch_abs_offset(bus, self.registers.y),
            AddrMode::Imm0 => self.dispatch_imm0(bus),
            AddrMode::Imp0 => self.dispatch_imp0(bus),
            AddrMode::Ind0 => self.dispatch_ind0(bus),
            AddrMode::IndX => self.dispatch_indx(bus),
            AddrMode::IndY => self.dispatch_indy(bus),
            AddrMode::Rel0 => self.dispatch_rel0(bus),
            AddrMode::Zpg0 => self.dispatch_zpg0(bus),
            AddrMode::ZpgX => self.dispatch_zpg_offset(bus, self.registers.x),
            AddrMode::ZpgY => self.dispatch_zpg_offset(bus, self.registers.y),
        }
    }
}
