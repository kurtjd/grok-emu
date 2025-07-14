mod flags;
mod instructions;
mod opcodes;

use grok_bus::BusHandler;
pub use opcodes::*;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

type MicroOp<T> = fn(&mut Cpu<T>, &mut T);
type TCycles = u64;
type MCycles = u64;

#[cfg(not(any(
    feature = "i8080",
    feature = "i8085",
    feature = "z80",
    feature = "sm83"
)))]
compile_error!("A CPU variant must be selected!");

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

bitflags::bitflags! {
    #[derive(PartialEq)]
    struct Flags: u8 {
        // Carry
        const CY = 0b00000010;
        // Parity
        const P  = 0b00000100;
        // Auxiliary carry
        const AC = 0b00001000;
        // Zero
        const Z  = 0b00010000;
        // Sign
        const S  = 0b00100000;
    }
}

pub struct Cycles {
    pub tcycles: TCycles,
    pub mcycles: MCycles,
}

#[derive(PartialEq, Eq)]
enum IffState {
    Disabled,
    EnablePending,
    Enabled,
}

pub struct Cpu<T: BusHandler> {
    // Program counter
    pc: u16,
    // Stack pointer
    sp: u16,
    // General purpose registers (including accumulator)
    gpr: [u8; 8],
    // Special 5-bit status flags register
    flags: Flags,
    // Hidden, temporary work registers
    wz: [u8; 2],
    // Interrupt enable state
    iff: IffState,
    // Pending interrupt opcode (almost always a RST_N)
    interrupt: Option<Opcode>,
    // Halt state
    halt: bool,
    // A "pipeline" of micro ops (each representing an M-cycle)
    pipeline: VecDeque<MicroOp<T>>,
    // Current opcode being executed
    op_info: OpcodeInfo,
    // Current M-cycle being executed
    mcycle: usize,
}

impl<T: BusHandler> Default for Cpu<T> {
    fn default() -> Self {
        Cpu::new()
    }
}

impl<T: BusHandler> Cpu<T> {
    /// Creates a new CPU
    pub fn new() -> Self {
        Self {
            pc: 0x0000,
            sp: 0x0000,
            gpr: [0x00; 8],
            flags: Flags::empty(),
            wz: [0x00; 2],
            iff: IffState::Disabled,
            interrupt: None,
            halt: false,
            pipeline: VecDeque::new(),
            op_info: Opcode::UNDEF_1.info(),
            mcycle: 0,
        }
    }

    /// Advances the CPU one M-cycle and returns the number of T-cycles taken
    pub fn tick(&mut self, bus: &mut T) -> Option<TCycles> {
        // If the pipeline is not empty, then work our way through it.
        if let Some(micro_op) = self.pipeline.pop_front() {
            micro_op(self, bus);

        // If the pipeline is empty, we know we've finished executing the previous instruction.
        // So fetch the next one.
        } else {
            let opcode = self.fetch(bus)?;

            // Executing the instruction really means it pushes its micro ops to the pipeline.
            // Though instructions consisting of a single M-cycle will also just immediately execute here.
            self.execute(opcode);
        }

        let tcycles = self.op_info.t_per_m[self.mcycle];
        self.mcycle += 1;
        tcycles
    }

    /// Advances the CPU one instruction and returns the number of M and T cycles taken
    pub fn step(&mut self, bus: &mut T) -> Option<Cycles> {
        // Should only be called on instruction boundaries
        assert!(self.pipeline.is_empty());

        // Fetch the next instruction
        let opcode = self.fetch(bus)?;

        self.execute(opcode);

        // Execute every micro op in the pipeline tracking total number of M and T cycles
        let mut tcycles = 0;
        while let Some(micro_op) = self.pipeline.pop_front() {
            micro_op(self, bus);
            tcycles += self.op_info.t_per_m[self.mcycle]?;
            self.mcycle += 1;
        }

        let mcycles = self.mcycle as u64;
        Some(Cycles { tcycles, mcycles })
    }

    /// Resets CPU setting PC to addr
    pub fn reset(&mut self, addr: u16) {
        self.pipeline.clear();
        self.mcycle = 0;
        self.halt = false;
        self.iff = IffState::Disabled;
        self.interrupt = None;
        self.pc = addr;
    }

    /// Pends an interrupt
    pub fn interrupt(&mut self, opcode: Opcode) {
        self.interrupt = Some(opcode);
    }

    fn fetch(&mut self, bus: &mut T) -> Option<Opcode> {
        // If interrupts are enabled and an interrupt is pending, handle it
        let opcode = if self.iff == IffState::Enabled
            && let Some(opcode) = self.interrupt
        {
            self.iff = IffState::Disabled;
            self.interrupt = None;
            self.halt = false;
            Some(opcode)

        // If halted, then do nothing
        } else if self.halt {
            None

        // Otherwise fetch next opcode as normal
        } else {
            // Need a 1 instruction delay between EI and interrupts actually being enabled
            if self.iff == IffState::EnablePending {
                self.iff = IffState::Enabled;
            }

            let opcode = Opcode::from(bus.mem_read(self.pc));
            self.pc = self.pc.wrapping_add(1);
            Some(opcode)
        };

        self.mcycle = 0;
        self.op_info = opcode?.info();
        opcode
    }

    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            // Data Transfer Group
            Opcode::MOV_A_A => self.mov_r_r(Register::A, Register::A),
            Opcode::MOV_A_B => self.mov_r_r(Register::A, Register::B),
            Opcode::MOV_A_C => self.mov_r_r(Register::A, Register::C),
            Opcode::MOV_A_D => self.mov_r_r(Register::A, Register::D),
            Opcode::MOV_A_E => self.mov_r_r(Register::A, Register::E),
            Opcode::MOV_A_H => self.mov_r_r(Register::A, Register::H),
            Opcode::MOV_A_L => self.mov_r_r(Register::A, Register::L),
            Opcode::MOV_A_M => self.mov_a_m(),
            Opcode::MOV_B_A => self.mov_r_r(Register::B, Register::A),
            Opcode::MOV_B_B => self.mov_r_r(Register::B, Register::B),
            Opcode::MOV_B_C => self.mov_r_r(Register::B, Register::C),
            Opcode::MOV_B_D => self.mov_r_r(Register::B, Register::D),
            Opcode::MOV_B_E => self.mov_r_r(Register::B, Register::E),
            Opcode::MOV_B_H => self.mov_r_r(Register::B, Register::H),
            Opcode::MOV_B_L => self.mov_r_r(Register::B, Register::L),
            Opcode::MOV_B_M => self.mov_b_m(),
            Opcode::MOV_C_A => self.mov_r_r(Register::C, Register::A),
            Opcode::MOV_C_B => self.mov_r_r(Register::C, Register::B),
            Opcode::MOV_C_C => self.mov_r_r(Register::C, Register::C),
            Opcode::MOV_C_D => self.mov_r_r(Register::C, Register::D),
            Opcode::MOV_C_E => self.mov_r_r(Register::C, Register::E),
            Opcode::MOV_C_H => self.mov_r_r(Register::C, Register::H),
            Opcode::MOV_C_L => self.mov_r_r(Register::C, Register::L),
            Opcode::MOV_C_M => self.mov_c_m(),
            Opcode::MOV_D_A => self.mov_r_r(Register::D, Register::A),
            Opcode::MOV_D_B => self.mov_r_r(Register::D, Register::B),
            Opcode::MOV_D_C => self.mov_r_r(Register::D, Register::C),
            Opcode::MOV_D_D => self.mov_r_r(Register::D, Register::D),
            Opcode::MOV_D_E => self.mov_r_r(Register::D, Register::E),
            Opcode::MOV_D_H => self.mov_r_r(Register::D, Register::H),
            Opcode::MOV_D_L => self.mov_r_r(Register::D, Register::L),
            Opcode::MOV_D_M => self.mov_d_m(),
            Opcode::MOV_E_A => self.mov_r_r(Register::E, Register::A),
            Opcode::MOV_E_B => self.mov_r_r(Register::E, Register::B),
            Opcode::MOV_E_C => self.mov_r_r(Register::E, Register::C),
            Opcode::MOV_E_D => self.mov_r_r(Register::E, Register::D),
            Opcode::MOV_E_E => self.mov_r_r(Register::E, Register::E),
            Opcode::MOV_E_H => self.mov_r_r(Register::E, Register::H),
            Opcode::MOV_E_L => self.mov_r_r(Register::E, Register::L),
            Opcode::MOV_E_M => self.mov_e_m(),
            Opcode::MOV_H_A => self.mov_r_r(Register::H, Register::A),
            Opcode::MOV_H_B => self.mov_r_r(Register::H, Register::B),
            Opcode::MOV_H_C => self.mov_r_r(Register::H, Register::C),
            Opcode::MOV_H_D => self.mov_r_r(Register::H, Register::D),
            Opcode::MOV_H_E => self.mov_r_r(Register::H, Register::E),
            Opcode::MOV_H_H => self.mov_r_r(Register::H, Register::H),
            Opcode::MOV_H_L => self.mov_r_r(Register::H, Register::L),
            Opcode::MOV_H_M => self.mov_h_m(),
            Opcode::MOV_L_A => self.mov_r_r(Register::L, Register::A),
            Opcode::MOV_L_B => self.mov_r_r(Register::L, Register::B),
            Opcode::MOV_L_C => self.mov_r_r(Register::L, Register::C),
            Opcode::MOV_L_D => self.mov_r_r(Register::L, Register::D),
            Opcode::MOV_L_E => self.mov_r_r(Register::L, Register::E),
            Opcode::MOV_L_H => self.mov_r_r(Register::L, Register::H),
            Opcode::MOV_L_L => self.mov_r_r(Register::L, Register::L),
            Opcode::MOV_L_M => self.mov_l_m(),
            Opcode::MOV_M_A => self.mov_m_a(),
            Opcode::MOV_M_B => self.mov_m_b(),
            Opcode::MOV_M_C => self.mov_m_c(),
            Opcode::MOV_M_D => self.mov_m_d(),
            Opcode::MOV_M_E => self.mov_m_e(),
            Opcode::MOV_M_H => self.mov_m_h(),
            Opcode::MOV_M_L => self.mov_m_l(),
            Opcode::MVI_A => self.mvi_a(),
            Opcode::MVI_B => self.mvi_b(),
            Opcode::MVI_C => self.mvi_c(),
            Opcode::MVI_D => self.mvi_d(),
            Opcode::MVI_E => self.mvi_e(),
            Opcode::MVI_H => self.mvi_h(),
            Opcode::MVI_L => self.mvi_l(),
            Opcode::MVI_M => self.mvi_m(),
            Opcode::LXI_B => self.lxi_b(),
            Opcode::LXI_D => self.lxi_d(),
            Opcode::LXI_H => self.lxi_h(),
            Opcode::LXI_SP => self.lxi_sp(),
            Opcode::LDA => self.lda(),
            Opcode::STA => self.sta(),
            Opcode::LHLD => self.lhld(),
            Opcode::SHLD => self.shld(),
            Opcode::LDAX_B => self.ldax_b(),
            Opcode::LDAX_D => self.ldax_d(),
            Opcode::STAX_B => self.stax_b(),
            Opcode::STAX_D => self.stax_d(),
            Opcode::XCHG => self.xchg(),
            // Arithmetic Group
            Opcode::ADD_A => self.add_r(Register::A),
            Opcode::ADD_B => self.add_r(Register::B),
            Opcode::ADD_C => self.add_r(Register::C),
            Opcode::ADD_D => self.add_r(Register::D),
            Opcode::ADD_E => self.add_r(Register::E),
            Opcode::ADD_H => self.add_r(Register::H),
            Opcode::ADD_L => self.add_r(Register::L),
            Opcode::ADD_M => self.add_m(),
            Opcode::ADI => self.adi(),
            Opcode::ADC_A => self.adc_r(Register::A),
            Opcode::ADC_B => self.adc_r(Register::B),
            Opcode::ADC_C => self.adc_r(Register::C),
            Opcode::ADC_D => self.adc_r(Register::D),
            Opcode::ADC_E => self.adc_r(Register::E),
            Opcode::ADC_H => self.adc_r(Register::H),
            Opcode::ADC_L => self.adc_r(Register::L),
            Opcode::ADC_M => self.adc_m(),
            Opcode::ACI => self.aci(),
            Opcode::SUB_A => self.sub_r(Register::A),
            Opcode::SUB_B => self.sub_r(Register::B),
            Opcode::SUB_C => self.sub_r(Register::C),
            Opcode::SUB_D => self.sub_r(Register::D),
            Opcode::SUB_E => self.sub_r(Register::E),
            Opcode::SUB_H => self.sub_r(Register::H),
            Opcode::SUB_L => self.sub_r(Register::L),
            Opcode::SUB_M => self.sub_m(),
            Opcode::SUI => self.sui(),
            Opcode::SBB_A => self.sbb_r(Register::A),
            Opcode::SBB_B => self.sbb_r(Register::B),
            Opcode::SBB_C => self.sbb_r(Register::C),
            Opcode::SBB_D => self.sbb_r(Register::D),
            Opcode::SBB_E => self.sbb_r(Register::E),
            Opcode::SBB_H => self.sbb_r(Register::H),
            Opcode::SBB_L => self.sbb_r(Register::L),
            Opcode::SBB_M => self.sbb_m(),
            Opcode::SBI => self.sbi(),
            Opcode::INR_A => self.inr_r(Register::A),
            Opcode::INR_B => self.inr_r(Register::B),
            Opcode::INR_C => self.inr_r(Register::C),
            Opcode::INR_D => self.inr_r(Register::D),
            Opcode::INR_E => self.inr_r(Register::E),
            Opcode::INR_H => self.inr_r(Register::H),
            Opcode::INR_L => self.inr_r(Register::L),
            Opcode::INR_M => self.inr_m(),
            Opcode::DCR_A => self.dcr_r(Register::A),
            Opcode::DCR_B => self.dcr_r(Register::B),
            Opcode::DCR_C => self.dcr_r(Register::C),
            Opcode::DCR_D => self.dcr_r(Register::D),
            Opcode::DCR_E => self.dcr_r(Register::E),
            Opcode::DCR_H => self.dcr_r(Register::H),
            Opcode::DCR_L => self.dcr_r(Register::L),
            Opcode::DCR_M => self.dcr_m(),
            Opcode::INX_B => self.inx_r(Register::B, Register::C),
            Opcode::INX_D => self.inx_r(Register::D, Register::E),
            Opcode::INX_H => self.inx_r(Register::H, Register::L),
            Opcode::INX_SP => self.inx_sp(),
            Opcode::DCX_B => self.dcx_r(Register::B, Register::C),
            Opcode::DCX_D => self.dcx_r(Register::D, Register::E),
            Opcode::DCX_H => self.dcx_r(Register::H, Register::L),
            Opcode::DCX_SP => self.dcx_sp(),
            Opcode::DAD_B => self.dad_b(),
            Opcode::DAD_D => self.dad_d(),
            Opcode::DAD_H => self.dad_h(),
            Opcode::DAD_SP => self.dad_sp(),
            Opcode::DAA => self.daa(),
            // Logical Group
            Opcode::ANA_A => self.ana_r(Register::A),
            Opcode::ANA_B => self.ana_r(Register::B),
            Opcode::ANA_C => self.ana_r(Register::C),
            Opcode::ANA_D => self.ana_r(Register::D),
            Opcode::ANA_E => self.ana_r(Register::E),
            Opcode::ANA_H => self.ana_r(Register::H),
            Opcode::ANA_L => self.ana_r(Register::L),
            Opcode::ANA_M => self.ana_m(),
            Opcode::ANI => self.ani(),
            Opcode::XRA_A => self.xra_r(Register::A),
            Opcode::XRA_B => self.xra_r(Register::B),
            Opcode::XRA_C => self.xra_r(Register::C),
            Opcode::XRA_D => self.xra_r(Register::D),
            Opcode::XRA_E => self.xra_r(Register::E),
            Opcode::XRA_H => self.xra_r(Register::H),
            Opcode::XRA_L => self.xra_r(Register::L),
            Opcode::XRA_M => self.xra_m(),
            Opcode::XRI => self.xri(),
            Opcode::ORA_A => self.ora_r(Register::A),
            Opcode::ORA_B => self.ora_r(Register::B),
            Opcode::ORA_C => self.ora_r(Register::C),
            Opcode::ORA_D => self.ora_r(Register::D),
            Opcode::ORA_E => self.ora_r(Register::E),
            Opcode::ORA_H => self.ora_r(Register::H),
            Opcode::ORA_L => self.ora_r(Register::L),
            Opcode::ORA_M => self.ora_m(),
            Opcode::ORI => self.ori(),
            Opcode::CMP_A => self.cmp_r(Register::A),
            Opcode::CMP_B => self.cmp_r(Register::B),
            Opcode::CMP_C => self.cmp_r(Register::C),
            Opcode::CMP_D => self.cmp_r(Register::D),
            Opcode::CMP_E => self.cmp_r(Register::E),
            Opcode::CMP_H => self.cmp_r(Register::H),
            Opcode::CMP_L => self.cmp_r(Register::L),
            Opcode::CMP_M => self.cmp_m(),
            Opcode::CPI => self.cpi(),
            Opcode::RLC => self.rlc(),
            Opcode::RRC => self.rrc(),
            Opcode::RAL => self.ral(),
            Opcode::RAR => self.rar(),
            Opcode::CMA => self.cma(),
            Opcode::CMC => self.cmc(),
            Opcode::STC => self.stc(),
            // Branch Group
            Opcode::JMP => self.jmp(),
            Opcode::JNZ => self.jnz(),
            Opcode::JZ => self.jz(),
            Opcode::JNC => self.jnc(),
            Opcode::JC => self.jc(),
            Opcode::JPO => self.jpo(),
            Opcode::JPE => self.jpe(),
            Opcode::JP => self.jp(),
            Opcode::JM => self.jm(),
            Opcode::CALL => self.call(),
            Opcode::CNZ => self.cnz(),
            Opcode::CZ => self.cz(),
            Opcode::CNC => self.cnc(),
            Opcode::CC => self.cc(),
            Opcode::CPO => self.cpo(),
            Opcode::CPE => self.cpe(),
            Opcode::CP => self.cp(),
            Opcode::CM => self.cm(),
            Opcode::RET => self.ret(),
            Opcode::RNZ => self.rnz(),
            Opcode::RZ => self.rz(),
            Opcode::RNC => self.rnc(),
            Opcode::RC => self.rc(),
            Opcode::RPO => self.rpo(),
            Opcode::RPE => self.rpe(),
            Opcode::RP => self.rp(),
            Opcode::RM => self.rm(),
            Opcode::RST_0 => self.rst_0(),
            Opcode::RST_1 => self.rst_1(),
            Opcode::RST_2 => self.rst_2(),
            Opcode::RST_3 => self.rst_3(),
            Opcode::RST_4 => self.rst_4(),
            Opcode::RST_5 => self.rst_5(),
            Opcode::RST_6 => self.rst_6(),
            Opcode::RST_7 => self.rst_7(),
            Opcode::PCHL => self.pchl(),
            // Stack Group
            Opcode::PUSH_B => self.push_b(),
            Opcode::PUSH_D => self.push_d(),
            Opcode::PUSH_H => self.push_h(),
            Opcode::PUSH_PSW => self.push_psw(),
            Opcode::POP_B => self.pop_b(),
            Opcode::POP_D => self.pop_d(),
            Opcode::POP_H => self.pop_h(),
            Opcode::POP_PSW => self.pop_psw(),
            Opcode::XTHL => self.xthl(),
            Opcode::SPHL => self.sphl(),
            // IO Group
            Opcode::IN => self.inp(),
            Opcode::OUT => self.outp(),
            // Machine Group
            Opcode::EI => self.ei(),
            Opcode::DI => self.di(),
            Opcode::HLT => self.hlt(),
            Opcode::NOP => self.nop(),
            #[cfg(feature = "i8085")]
            Opcode::RIM => todo!(),
            #[cfg(feature = "i8085")]
            Opcode::SIM => todo!(),
            // Undefined
            Opcode::UNDEF_1 => self.undef(opcode),
            Opcode::UNDEF_2 => self.undef(opcode),
            Opcode::UNDEF_3 => self.undef(opcode),
            #[cfg(feature = "i8080")]
            Opcode::UNDEF_4 => self.undef(opcode),
            Opcode::UNDEF_5 => self.undef(opcode),
            #[cfg(feature = "i8080")]
            Opcode::UNDEF_6 => self.undef(opcode),
            Opcode::UNDEF_7 => self.undef(opcode),
            Opcode::UNDEF_8 => self.undef(opcode),
            Opcode::UNDEF_9 => self.undef(opcode),
            Opcode::UNDEF_10 => self.undef(opcode),
            Opcode::UNDEF_11 => self.undef(opcode),
            Opcode::UNDEF_12 => self.undef(opcode),
        }
    }

    fn get_reg_pair(&self, reg1: Register, reg2: Register) -> u16 {
        u16::from_be_bytes([self.gpr[reg1], self.gpr[reg2]])
    }

    fn set_reg_pair(&mut self, reg1: Register, reg2: Register, val: u16) {
        let [high, low] = val.to_be_bytes();
        self.gpr[reg1] = high;
        self.gpr[reg2] = low;
    }

    fn get_psw(&self) -> u16 {
        u16::from_be_bytes([self.gpr[Register::A], self.flags_to_u8()])
    }

    fn flags_to_u8(&self) -> u8 {
        let flag = |f| self.flags.contains(f) as u8;
        (flag(Flags::S) << 7)
            | (flag(Flags::Z) << 6)
            | (flag(Flags::AC) << 4)
            | (flag(Flags::P) << 2)
            | (1 << 1)
            | (flag(Flags::CY))
    }

    fn flags_to_str(&self) -> String {
        self.flags
            .iter()
            .map(|f| match f {
                Flags::AC => "AC",
                Flags::CY => "CY",
                Flags::P => "P",
                Flags::S => "S",
                Flags::Z => "Z",
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// Saves us from needing to cast to usize everywhere
impl Index<Register> for [u8] {
    type Output = u8;

    fn index(&self, index: Register) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Register> for [u8] {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T: BusHandler> grok_dbg::DebugHandler<T> for Cpu<T> {
    fn peek(&mut self, bus: &mut T, addr: usize) -> u8 {
        bus.mem_peek(addr as u16)
    }

    fn print_debug(&mut self, bus: &mut T) {
        let opcode = Opcode::from(bus.mem_peek(self.pc));
        let opcode_info = opcode.info();

        let (op_asm, op_bytes) = if opcode_info.len == 1 {
            ("".to_string(), "".to_string())
        } else if opcode_info.len == 2 {
            let val = bus.mem_peek(self.pc + 1);
            (format!("{val:02X}H"), format!(" {val:02X}"))
        } else {
            let val1 = bus.mem_peek(self.pc + 1);
            let val2 = bus.mem_peek(self.pc + 2);
            (
                format!("{:04X}H", u16::from_le_bytes([val1, val2,])),
                format!(" {val1:02X} {val2:02X}"),
            )
        };

        println!(
            "{:04X}: {:02X}{} ({}{})",
            self.pc, opcode as u8, op_bytes, opcode_info.name, op_asm
        );

        println!(
            "A={:02X}  BC={:04X}  DE={:04X}  HL={:04X}  SP={:04X}  PSW={:04X}  FLAGS={:02X}:[{}]",
            self.gpr[Register::A],
            self.get_reg_pair(Register::B, Register::C),
            self.get_reg_pair(Register::D, Register::E),
            self.get_reg_pair(Register::H, Register::L),
            self.sp,
            self.get_psw(),
            self.flags.bits(),
            self.flags_to_str(),
        );
    }

    fn step(&mut self, bus: &mut T) -> usize {
        self.step(bus);
        self.pc as usize
    }
}
