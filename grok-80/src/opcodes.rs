use crate::{BusHandlerZ80, Cpu, Flags, Reg, RegPair, TCycles, opcodes_cb::OpcodeCB};

enum AluOp {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Or,
    Xor,
    Cp,
}

#[derive(Copy, Clone, Debug, num_enum::FromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Opcode {
    NOP,
    LD_BC_NN,
    LD_BCi_A,
    INC_BC,
    INC_B,
    DEC_B,
    LD_B_N,
    RLCA,
    EX_AF_AF,
    ADD_HL_BC,
    LD_A_BCi,
    DEC_BC,
    INC_C,
    DEC_C,
    LD_C_N,
    RRCA,
    DJNZ_DIS,
    LD_DE_NN,
    LD_DEi_A,
    INC_DE,
    INC_D,
    DEC_D,
    LD_D_N,
    RLA,
    JR_DIS,
    ADD_HL_DE,
    LD_A_DEi,
    DEC_DE,
    INC_E,
    DEC_E,
    LD_E_N,
    RRA,
    JR_NZ_DIS,
    LD_HL_NN,
    LD_NNi_HL,
    INC_HL,
    INC_H,
    DEC_H,
    LD_H_N,
    DAA,
    JR_Z_DIS,
    ADD_HL_HL,
    LD_HL_NNi,
    DEC_HL,
    INC_L,
    DEC_L,
    LD_L_N,
    CPL,
    JR_NC_DIS,
    LD_SP_NN,
    LD_NNi_A,
    INC_SP,
    INC_HLi,
    DEC_HLi,
    LD_HLi_N,
    SCF,
    JR_C_DIS,
    ADD_HL_SP,
    LD_A_NNi,
    DEC_SP,
    INC_A,
    DEC_A,
    LD_A_N,
    CCF,
    LD_B_B,
    LD_B_C,
    LD_B_D,
    LD_B_E,
    LD_B_H,
    LD_B_L,
    LD_B_HLi,
    LD_B_A,
    LD_C_B,
    LD_C_C,
    LD_C_D,
    LD_C_E,
    LD_C_H,
    LD_C_L,
    LD_C_HLi,
    LD_C_A,
    LD_D_B,
    LD_D_C,
    LD_D_D,
    LD_D_E,
    LD_D_H,
    LD_D_L,
    LD_D_HLi,
    LD_D_A,
    LD_E_B,
    LD_E_C,
    LD_E_D,
    LD_E_E,
    LD_E_H,
    LD_E_L,
    LD_E_HLi,
    LD_E_A,
    LD_H_B,
    LD_H_C,
    LD_H_D,
    LD_H_E,
    LD_H_H,
    LD_H_L,
    LD_H_HLi,
    LD_H_A,
    LD_L_B,
    LD_L_C,
    LD_L_D,
    LD_L_E,
    LD_L_H,
    LD_L_L,
    LD_L_HLi,
    LD_L_A,
    LD_HLi_B,
    LD_HLi_C,
    LD_HLi_D,
    LD_HLi_E,
    LD_HLi_H,
    LD_HLi_L,
    HALT,
    LD_HLi_A,
    LD_A_B,
    LD_A_C,
    LD_A_D,
    LD_A_E,
    LD_A_H,
    LD_A_L,
    LD_A_HLi,
    LD_A_A,
    ADD_A_B,
    ADD_A_C,
    ADD_A_D,
    ADD_A_E,
    ADD_A_H,
    ADD_A_L,
    ADD_A_HLi,
    ADD_A_A,
    ADC_A_B,
    ADC_A_C,
    ADC_A_D,
    ADC_A_E,
    ADC_A_H,
    ADC_A_L,
    ADC_A_HLi,
    ADC_A_A,
    SUB_B,
    SUB_C,
    SUB_D,
    SUB_E,
    SUB_H,
    SUB_L,
    SUB_HLi,
    SUB_A,
    SBC_A_B,
    SBC_A_C,
    SBC_A_D,
    SBC_A_E,
    SBC_A_H,
    SBC_A_L,
    SBC_A_HLi,
    SBC_A_A,
    AND_B,
    AND_C,
    AND_D,
    AND_E,
    AND_H,
    AND_L,
    AND_HLi,
    AND_A,
    XOR_B,
    XOR_C,
    XOR_D,
    XOR_E,
    XOR_H,
    XOR_L,
    XOR_HLi,
    XOR_A,
    OR_B,
    OR_C,
    OR_D,
    OR_E,
    OR_H,
    OR_L,
    OR_HLi,
    OR_A,
    CP_B,
    CP_C,
    CP_D,
    CP_E,
    CP_H,
    CP_L,
    CP_HLi,
    CP_A,
    RET_NZ,
    POP_BC,
    JP_NZ_NN,
    JP_NN,
    CALL_NZ_NN,
    PUSH_BC,
    ADD_A_N,
    RST_00H,
    RET_Z,
    RET,
    JP_Z_NN,
    PREFIX_CB,
    CALL_Z_NN,
    CALL_NN,
    ADC_A_N,
    RST_08H,
    RET_NC,
    POP_DE,
    JP_NC_NN,
    OUT_N_A,
    CALL_NC_NN,
    PUSH_DE,
    SUB_N,
    RST_10H,
    RET_C,
    EXX,
    JP_C_NN,
    IN_A_N,
    CALL_C_NN,
    PREFIX_DD,
    SBC_A_N,
    RST_18H,
    RET_PO,
    POP_HL,
    JP_PO_NN,
    EX_SPi_HL,
    CALL_PO_NN,
    PUSH_HL,
    AND_N,
    RST_20H,
    RET_PE,
    JP_HLi,
    JP_PE_NN,
    EX_DE_HL,
    CALL_PE_NN,
    PREFIX_ED,
    XOR_N,
    RST_28H,
    RET_P,
    POP_AF,
    JP_P_NN,
    DI,
    CALL_P_NN,
    PUSH_AF,
    OR_N,
    RST_30H,
    RET_M,
    LD_SP_HL,
    JP_M_NN,
    EI,
    CALL_M_NN,
    PREFIX_FD,
    CP_N,
    RST_38H,
}

impl<B: BusHandlerZ80> Cpu<B> {
    pub(crate) fn execute(&mut self, opcode: Opcode, bus: &mut B) {
        match opcode {
            // 8-Bit Load Group
            Opcode::LD_A_A => self.ld_r_r(bus, Reg::A, Reg::A),
            Opcode::LD_A_B => self.ld_r_r(bus, Reg::A, Reg::B),
            Opcode::LD_A_C => self.ld_r_r(bus, Reg::A, Reg::C),
            Opcode::LD_A_D => self.ld_r_r(bus, Reg::A, Reg::D),
            Opcode::LD_A_E => self.ld_r_r(bus, Reg::A, Reg::E),
            Opcode::LD_A_H => self.ld_r_r(bus, Reg::A, Reg::H),
            Opcode::LD_A_L => self.ld_r_r(bus, Reg::A, Reg::L),
            Opcode::LD_B_A => self.ld_r_r(bus, Reg::B, Reg::A),
            Opcode::LD_B_B => self.ld_r_r(bus, Reg::B, Reg::B),
            Opcode::LD_B_C => self.ld_r_r(bus, Reg::B, Reg::C),
            Opcode::LD_B_D => self.ld_r_r(bus, Reg::B, Reg::D),
            Opcode::LD_B_E => self.ld_r_r(bus, Reg::B, Reg::E),
            Opcode::LD_B_H => self.ld_r_r(bus, Reg::B, Reg::H),
            Opcode::LD_B_L => self.ld_r_r(bus, Reg::B, Reg::L),
            Opcode::LD_C_A => self.ld_r_r(bus, Reg::C, Reg::A),
            Opcode::LD_C_B => self.ld_r_r(bus, Reg::C, Reg::B),
            Opcode::LD_C_C => self.ld_r_r(bus, Reg::C, Reg::C),
            Opcode::LD_C_D => self.ld_r_r(bus, Reg::C, Reg::D),
            Opcode::LD_C_E => self.ld_r_r(bus, Reg::C, Reg::E),
            Opcode::LD_C_H => self.ld_r_r(bus, Reg::C, Reg::H),
            Opcode::LD_C_L => self.ld_r_r(bus, Reg::C, Reg::L),
            Opcode::LD_D_A => self.ld_r_r(bus, Reg::D, Reg::A),
            Opcode::LD_D_B => self.ld_r_r(bus, Reg::D, Reg::B),
            Opcode::LD_D_C => self.ld_r_r(bus, Reg::D, Reg::C),
            Opcode::LD_D_D => self.ld_r_r(bus, Reg::D, Reg::D),
            Opcode::LD_D_E => self.ld_r_r(bus, Reg::D, Reg::E),
            Opcode::LD_D_H => self.ld_r_r(bus, Reg::D, Reg::H),
            Opcode::LD_D_L => self.ld_r_r(bus, Reg::D, Reg::L),
            Opcode::LD_E_A => self.ld_r_r(bus, Reg::E, Reg::A),
            Opcode::LD_E_B => self.ld_r_r(bus, Reg::E, Reg::B),
            Opcode::LD_E_C => self.ld_r_r(bus, Reg::E, Reg::C),
            Opcode::LD_E_D => self.ld_r_r(bus, Reg::E, Reg::D),
            Opcode::LD_E_E => self.ld_r_r(bus, Reg::E, Reg::E),
            Opcode::LD_E_H => self.ld_r_r(bus, Reg::E, Reg::H),
            Opcode::LD_E_L => self.ld_r_r(bus, Reg::E, Reg::L),
            Opcode::LD_H_A => self.ld_r_r(bus, Reg::H, Reg::A),
            Opcode::LD_H_B => self.ld_r_r(bus, Reg::H, Reg::B),
            Opcode::LD_H_C => self.ld_r_r(bus, Reg::H, Reg::C),
            Opcode::LD_H_D => self.ld_r_r(bus, Reg::H, Reg::D),
            Opcode::LD_H_E => self.ld_r_r(bus, Reg::H, Reg::E),
            Opcode::LD_H_H => self.ld_r_r(bus, Reg::H, Reg::H),
            Opcode::LD_H_L => self.ld_r_r(bus, Reg::H, Reg::L),
            Opcode::LD_L_A => self.ld_r_r(bus, Reg::L, Reg::A),
            Opcode::LD_L_B => self.ld_r_r(bus, Reg::L, Reg::B),
            Opcode::LD_L_C => self.ld_r_r(bus, Reg::L, Reg::C),
            Opcode::LD_L_D => self.ld_r_r(bus, Reg::L, Reg::D),
            Opcode::LD_L_E => self.ld_r_r(bus, Reg::L, Reg::E),
            Opcode::LD_L_H => self.ld_r_r(bus, Reg::L, Reg::H),
            Opcode::LD_L_L => self.ld_r_r(bus, Reg::L, Reg::L),
            Opcode::LD_A_N => self.ld_r_n(bus, Reg::A),
            Opcode::LD_B_N => self.ld_r_n(bus, Reg::B),
            Opcode::LD_C_N => self.ld_r_n(bus, Reg::C),
            Opcode::LD_D_N => self.ld_r_n(bus, Reg::D),
            Opcode::LD_E_N => self.ld_r_n(bus, Reg::E),
            Opcode::LD_H_N => self.ld_r_n(bus, Reg::H),
            Opcode::LD_L_N => self.ld_r_n(bus, Reg::L),
            Opcode::LD_A_HLi => self.ld_r_rpi(bus, Reg::A, RegPair::HL),
            Opcode::LD_A_BCi => self.ld_r_rpi(bus, Reg::A, RegPair::BC),
            Opcode::LD_A_DEi => self.ld_r_rpi(bus, Reg::A, RegPair::DE),
            Opcode::LD_B_HLi => self.ld_r_rpi(bus, Reg::B, RegPair::HL),
            Opcode::LD_C_HLi => self.ld_r_rpi(bus, Reg::C, RegPair::HL),
            Opcode::LD_D_HLi => self.ld_r_rpi(bus, Reg::D, RegPair::HL),
            Opcode::LD_E_HLi => self.ld_r_rpi(bus, Reg::E, RegPair::HL),
            Opcode::LD_H_HLi => self.ld_r_rpi(bus, Reg::H, RegPair::HL),
            Opcode::LD_L_HLi => self.ld_r_rpi(bus, Reg::L, RegPair::HL),
            Opcode::LD_HLi_A => self.ld_rpi_r(bus, RegPair::HL, Reg::A),
            Opcode::LD_BCi_A => self.ld_rpi_r(bus, RegPair::BC, Reg::A),
            Opcode::LD_DEi_A => self.ld_rpi_r(bus, RegPair::DE, Reg::A),
            Opcode::LD_HLi_B => self.ld_rpi_r(bus, RegPair::HL, Reg::B),
            Opcode::LD_HLi_C => self.ld_rpi_r(bus, RegPair::HL, Reg::C),
            Opcode::LD_HLi_D => self.ld_rpi_r(bus, RegPair::HL, Reg::D),
            Opcode::LD_HLi_E => self.ld_rpi_r(bus, RegPair::HL, Reg::E),
            Opcode::LD_HLi_H => self.ld_rpi_r(bus, RegPair::HL, Reg::H),
            Opcode::LD_HLi_L => self.ld_rpi_r(bus, RegPair::HL, Reg::L),
            Opcode::LD_HLi_N => self.ld_hli_n(bus),
            Opcode::LD_A_NNi => self.ld_a_nni(bus),
            Opcode::LD_NNi_A => self.ld_nni_a(bus),

            // 8-Bit Arithmetic and Logic Group
            Opcode::ADD_A_A => self.alu_r(bus, Reg::A, AluOp::Add),
            Opcode::ADD_A_B => self.alu_r(bus, Reg::B, AluOp::Add),
            Opcode::ADD_A_C => self.alu_r(bus, Reg::C, AluOp::Add),
            Opcode::ADD_A_D => self.alu_r(bus, Reg::D, AluOp::Add),
            Opcode::ADD_A_E => self.alu_r(bus, Reg::E, AluOp::Add),
            Opcode::ADD_A_H => self.alu_r(bus, Reg::H, AluOp::Add),
            Opcode::ADD_A_L => self.alu_r(bus, Reg::L, AluOp::Add),
            Opcode::ADD_A_N => self.alu_n(bus, AluOp::Add),
            Opcode::ADD_A_HLi => self.alu_hli(bus, AluOp::Add),
            Opcode::ADC_A_A => self.alu_r(bus, Reg::A, AluOp::Adc),
            Opcode::ADC_A_B => self.alu_r(bus, Reg::B, AluOp::Adc),
            Opcode::ADC_A_C => self.alu_r(bus, Reg::C, AluOp::Adc),
            Opcode::ADC_A_D => self.alu_r(bus, Reg::D, AluOp::Adc),
            Opcode::ADC_A_E => self.alu_r(bus, Reg::E, AluOp::Adc),
            Opcode::ADC_A_H => self.alu_r(bus, Reg::H, AluOp::Adc),
            Opcode::ADC_A_L => self.alu_r(bus, Reg::L, AluOp::Adc),
            Opcode::ADC_A_N => self.alu_n(bus, AluOp::Adc),
            Opcode::ADC_A_HLi => self.alu_hli(bus, AluOp::Adc),
            Opcode::SUB_A => self.alu_r(bus, Reg::A, AluOp::Sub),
            Opcode::SUB_B => self.alu_r(bus, Reg::B, AluOp::Sub),
            Opcode::SUB_C => self.alu_r(bus, Reg::C, AluOp::Sub),
            Opcode::SUB_D => self.alu_r(bus, Reg::D, AluOp::Sub),
            Opcode::SUB_E => self.alu_r(bus, Reg::E, AluOp::Sub),
            Opcode::SUB_H => self.alu_r(bus, Reg::H, AluOp::Sub),
            Opcode::SUB_L => self.alu_r(bus, Reg::L, AluOp::Sub),
            Opcode::SUB_N => self.alu_n(bus, AluOp::Sub),
            Opcode::SUB_HLi => self.alu_hli(bus, AluOp::Sub),
            Opcode::SBC_A_A => self.alu_r(bus, Reg::A, AluOp::Sbc),
            Opcode::SBC_A_B => self.alu_r(bus, Reg::B, AluOp::Sbc),
            Opcode::SBC_A_C => self.alu_r(bus, Reg::C, AluOp::Sbc),
            Opcode::SBC_A_D => self.alu_r(bus, Reg::D, AluOp::Sbc),
            Opcode::SBC_A_E => self.alu_r(bus, Reg::E, AluOp::Sbc),
            Opcode::SBC_A_H => self.alu_r(bus, Reg::H, AluOp::Sbc),
            Opcode::SBC_A_L => self.alu_r(bus, Reg::L, AluOp::Sbc),
            Opcode::SBC_A_N => self.alu_n(bus, AluOp::Sbc),
            Opcode::SBC_A_HLi => self.alu_hli(bus, AluOp::Sbc),
            Opcode::AND_A => self.alu_r(bus, Reg::A, AluOp::And),
            Opcode::AND_B => self.alu_r(bus, Reg::B, AluOp::And),
            Opcode::AND_C => self.alu_r(bus, Reg::C, AluOp::And),
            Opcode::AND_D => self.alu_r(bus, Reg::D, AluOp::And),
            Opcode::AND_E => self.alu_r(bus, Reg::E, AluOp::And),
            Opcode::AND_H => self.alu_r(bus, Reg::H, AluOp::And),
            Opcode::AND_L => self.alu_r(bus, Reg::L, AluOp::And),
            Opcode::AND_N => self.alu_n(bus, AluOp::And),
            Opcode::AND_HLi => self.alu_hli(bus, AluOp::And),
            Opcode::OR_A => self.alu_r(bus, Reg::A, AluOp::Or),
            Opcode::OR_B => self.alu_r(bus, Reg::B, AluOp::Or),
            Opcode::OR_C => self.alu_r(bus, Reg::C, AluOp::Or),
            Opcode::OR_D => self.alu_r(bus, Reg::D, AluOp::Or),
            Opcode::OR_E => self.alu_r(bus, Reg::E, AluOp::Or),
            Opcode::OR_H => self.alu_r(bus, Reg::H, AluOp::Or),
            Opcode::OR_L => self.alu_r(bus, Reg::L, AluOp::Or),
            Opcode::OR_N => self.alu_n(bus, AluOp::Or),
            Opcode::OR_HLi => self.alu_hli(bus, AluOp::Or),
            Opcode::XOR_A => self.alu_r(bus, Reg::A, AluOp::Xor),
            Opcode::XOR_B => self.alu_r(bus, Reg::B, AluOp::Xor),
            Opcode::XOR_C => self.alu_r(bus, Reg::C, AluOp::Xor),
            Opcode::XOR_D => self.alu_r(bus, Reg::D, AluOp::Xor),
            Opcode::XOR_E => self.alu_r(bus, Reg::E, AluOp::Xor),
            Opcode::XOR_H => self.alu_r(bus, Reg::H, AluOp::Xor),
            Opcode::XOR_L => self.alu_r(bus, Reg::L, AluOp::Xor),
            Opcode::XOR_N => self.alu_n(bus, AluOp::Xor),
            Opcode::XOR_HLi => self.alu_hli(bus, AluOp::Xor),
            Opcode::CP_A => self.alu_r(bus, Reg::A, AluOp::Cp),
            Opcode::CP_B => self.alu_r(bus, Reg::B, AluOp::Cp),
            Opcode::CP_C => self.alu_r(bus, Reg::C, AluOp::Cp),
            Opcode::CP_D => self.alu_r(bus, Reg::D, AluOp::Cp),
            Opcode::CP_E => self.alu_r(bus, Reg::E, AluOp::Cp),
            Opcode::CP_H => self.alu_r(bus, Reg::H, AluOp::Cp),
            Opcode::CP_L => self.alu_r(bus, Reg::L, AluOp::Cp),
            Opcode::CP_N => self.alu_n(bus, AluOp::Cp),
            Opcode::CP_HLi => self.alu_hli(bus, AluOp::Cp),
            Opcode::INC_A => self.inc_dec_r(bus, Reg::A, true),
            Opcode::INC_B => self.inc_dec_r(bus, Reg::B, true),
            Opcode::INC_C => self.inc_dec_r(bus, Reg::C, true),
            Opcode::INC_D => self.inc_dec_r(bus, Reg::D, true),
            Opcode::INC_E => self.inc_dec_r(bus, Reg::E, true),
            Opcode::INC_H => self.inc_dec_r(bus, Reg::H, true),
            Opcode::INC_L => self.inc_dec_r(bus, Reg::L, true),
            Opcode::INC_HLi => self.inc_dec_hli(bus, true),
            Opcode::DEC_A => self.inc_dec_r(bus, Reg::A, false),
            Opcode::DEC_B => self.inc_dec_r(bus, Reg::B, false),
            Opcode::DEC_C => self.inc_dec_r(bus, Reg::C, false),
            Opcode::DEC_D => self.inc_dec_r(bus, Reg::D, false),
            Opcode::DEC_E => self.inc_dec_r(bus, Reg::E, false),
            Opcode::DEC_H => self.inc_dec_r(bus, Reg::H, false),
            Opcode::DEC_L => self.inc_dec_r(bus, Reg::L, false),
            Opcode::DEC_HLi => self.inc_dec_hli(bus, false),

            // General-Purpose Arithmetic and Logic Group
            Opcode::DAA => self.daa(bus),
            Opcode::CPL => self.cpl(bus),
            Opcode::CCF => self.ccf(bus),
            Opcode::SCF => self.scf(bus),

            // CPU Control Group
            Opcode::NOP => self.nop(bus),
            Opcode::HALT => self.halt(bus),
            Opcode::DI => self.ei(bus, false),
            Opcode::EI => self.ei(bus, true),

            // 16-Bit Arithmetic Group
            Opcode::ADD_HL_BC => self.add_hl_rr(bus, RegPair::BC),
            Opcode::ADD_HL_DE => self.add_hl_rr(bus, RegPair::DE),
            Opcode::ADD_HL_HL => self.add_hl_rr(bus, RegPair::HL),
            Opcode::ADD_HL_SP => self.add_hl_rr(bus, RegPair::SP),
            Opcode::INC_BC => self.inc_dec_rr(bus, RegPair::BC, true),
            Opcode::INC_DE => self.inc_dec_rr(bus, RegPair::DE, true),
            Opcode::INC_HL => self.inc_dec_rr(bus, RegPair::HL, true),
            Opcode::INC_SP => self.inc_dec_rr(bus, RegPair::SP, true),
            Opcode::DEC_BC => self.inc_dec_rr(bus, RegPair::BC, false),
            Opcode::DEC_DE => self.inc_dec_rr(bus, RegPair::DE, false),
            Opcode::DEC_HL => self.inc_dec_rr(bus, RegPair::HL, false),
            Opcode::DEC_SP => self.inc_dec_rr(bus, RegPair::SP, false),

            // Rotate and Shift Group
            Opcode::RLCA => self.ra(bus, true, false),
            Opcode::RLA => self.ra(bus, true, true),
            Opcode::RRCA => self.ra(bus, false, false),
            Opcode::RRA => self.ra(bus, false, true),

            // Jump Group
            Opcode::JP_NN => self.jp_nn(bus, true),
            Opcode::JP_NZ_NN => self.jp_nn(bus, !self.reg.f.contains(Flags::Z)),
            Opcode::JP_Z_NN => self.jp_nn(bus, self.reg.f.contains(Flags::Z)),
            Opcode::JP_NC_NN => self.jp_nn(bus, !self.reg.f.contains(Flags::C)),
            Opcode::JP_C_NN => self.jp_nn(bus, self.reg.f.contains(Flags::C)),
            Opcode::JP_PO_NN => self.jp_nn(bus, !self.reg.f.contains(Flags::P)),
            Opcode::JP_PE_NN => self.jp_nn(bus, self.reg.f.contains(Flags::P)),
            Opcode::JP_P_NN => self.jp_nn(bus, !self.reg.f.contains(Flags::S)),
            Opcode::JP_M_NN => self.jp_nn(bus, self.reg.f.contains(Flags::S)),
            Opcode::JR_DIS => self.jr_dis(bus, true),
            Opcode::JR_C_DIS => self.jr_dis(bus, self.reg.f.contains(Flags::C)),
            Opcode::JR_NC_DIS => self.jr_dis(bus, !self.reg.f.contains(Flags::C)),
            Opcode::JR_Z_DIS => self.jr_dis(bus, self.reg.f.contains(Flags::Z)),
            Opcode::JR_NZ_DIS => self.jr_dis(bus, !self.reg.f.contains(Flags::Z)),
            Opcode::JP_HLi => self.jp_hli(bus),
            Opcode::DJNZ_DIS => self.djnz_dis(bus),

            // Call and Return Group
            Opcode::CALL_NN => self.call(bus, true),
            Opcode::CALL_NZ_NN => self.call(bus, !self.reg.f.contains(Flags::Z)),
            Opcode::CALL_Z_NN => self.call(bus, self.reg.f.contains(Flags::Z)),
            Opcode::CALL_NC_NN => self.call(bus, !self.reg.f.contains(Flags::C)),
            Opcode::CALL_C_NN => self.call(bus, self.reg.f.contains(Flags::C)),
            Opcode::CALL_PO_NN => self.call(bus, !self.reg.f.contains(Flags::P)),
            Opcode::CALL_PE_NN => self.call(bus, self.reg.f.contains(Flags::P)),
            Opcode::CALL_P_NN => self.call(bus, !self.reg.f.contains(Flags::S)),
            Opcode::CALL_M_NN => self.call(bus, self.reg.f.contains(Flags::S)),
            Opcode::RET => self.ret(bus, 5),
            Opcode::RET_NZ => self.ret_cc(bus, !self.reg.f.contains(Flags::Z)),
            Opcode::RET_Z => self.ret_cc(bus, self.reg.f.contains(Flags::Z)),
            Opcode::RET_NC => self.ret_cc(bus, !self.reg.f.contains(Flags::C)),
            Opcode::RET_C => self.ret_cc(bus, self.reg.f.contains(Flags::C)),
            Opcode::RET_PO => self.ret_cc(bus, !self.reg.f.contains(Flags::P)),
            Opcode::RET_PE => self.ret_cc(bus, self.reg.f.contains(Flags::P)),
            Opcode::RET_P => self.ret_cc(bus, !self.reg.f.contains(Flags::S)),
            Opcode::RET_M => self.ret_cc(bus, self.reg.f.contains(Flags::S)),
            Opcode::RST_00H => self.rst(bus, 0x00),
            Opcode::RST_08H => self.rst(bus, 0x08),
            Opcode::RST_10H => self.rst(bus, 0x10),
            Opcode::RST_18H => self.rst(bus, 0x18),
            Opcode::RST_20H => self.rst(bus, 0x20),
            Opcode::RST_28H => self.rst(bus, 0x28),
            Opcode::RST_30H => self.rst(bus, 0x30),
            Opcode::RST_38H => self.rst(bus, 0x38),

            // Input and Output Group
            Opcode::IN_A_N => self.in_a_n(bus),
            Opcode::OUT_N_A => self.out_n_a(bus),

            Opcode::PREFIX_CB => match self.tcycle {
                4 => {}
                5 => self.fetch_t1(bus),
                6 => self.fetch_t2(bus),
                7 => {
                    self.reg.ir_pre = self.fetch_t3(bus);
                }
                8 => {
                    self.fetch_t4(bus);
                    self.execute_prefix_cb(OpcodeCB::from(self.reg.ir_pre), bus);
                }
                _ => self.execute_prefix_cb(OpcodeCB::from(self.reg.ir_pre), bus),
            },
            Opcode::EXX => {
                self.reg.exchange(Reg::B);
                self.reg.exchange(Reg::C);
                self.reg.exchange(Reg::D);
                self.reg.exchange(Reg::E);
                self.reg.exchange(Reg::H);
                self.reg.exchange(Reg::L);
                self.end_instruction(bus, false);
            }

            _ => todo!("Opcode: {:?}", opcode),
        }
    }

    // 8-Bit Load Group
    fn ld_r_r(&mut self, bus: &mut B, dst: Reg, src: Reg) {
        self.reg.set(dst, self.reg.get(src));
        self.end_instruction(bus, false);
    }

    fn ld_r_n(&mut self, bus: &mut B, dst: Reg) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                let data = self.mem_rd_t3(bus);
                self.reg.set(dst, data);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ld_r_rpi(&mut self, bus: &mut B, dst: Reg, src: RegPair) {
        match self.tcycle {
            5 => {
                let addr = self.reg.get_pair(src);
                self.mem_rd_t1(bus, addr);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                let data = self.mem_rd_t3(bus);
                self.reg.set(dst, data);

                // Special case for `LD A, (BC)` and `LD A, (DE)`
                if dst == Reg::A && (src == RegPair::BC || src == RegPair::DE) {
                    self.reg.wz = self.reg.get_pair(src) + 1;
                }

                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ld_rpi_r(&mut self, bus: &mut B, dst: RegPair, src: Reg) {
        match self.tcycle {
            5 => {
                let addr = self.reg.get_pair(dst);
                self.mem_wr_t1(bus, addr);
            }
            6 => {
                self.mem_wr_t2(bus, self.reg.get(src));
            }
            7 => {
                self.mem_wr_t3(bus);

                // Special case for `LD (BC), A` and `LD (DE), A`
                if src == Reg::A && (dst == RegPair::BC || dst == RegPair::DE) {
                    self.reg.wz =
                        u16::from_be_bytes([self.reg.a, (self.reg.get_pair(dst) + 1) as u8]);
                }

                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ld_hli_n(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                let addr = self.reg.get_pair(RegPair::HL);
                self.mem_wr_t1(bus, addr);
            }
            9 => {
                self.mem_wr_t2(bus, self.reg.tmp[0]);
            }
            10 => {
                self.mem_wr_t3(bus);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ld_a_nni(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.mem_rd_t1_imm(bus);
            }
            9 => {
                self.mem_rd_t2(bus);
            }
            10 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
            }
            11 => {
                let addr = u16::from_le_bytes(self.reg.tmp);
                self.mem_rd_t1(bus, addr);
                self.reg.wz = addr + 1;
            }
            12 => {
                self.mem_rd_t2(bus);
            }
            13 => {
                self.reg.a = self.mem_rd_t3(bus);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ld_nni_a(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.mem_rd_t1_imm(bus);
            }
            9 => {
                self.mem_rd_t2(bus);
            }
            10 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
            }
            11 => {
                let addr = u16::from_le_bytes(self.reg.tmp);
                self.mem_wr_t1(bus, addr);
                self.reg.wz = u16::from_be_bytes([self.reg.a, (addr + 1) as u8]);
            }
            12 => {
                self.mem_wr_t2(bus, self.reg.a);
            }
            13 => {
                self.mem_wr_t3(bus);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    // 8-Bit Arithmetic and Logic Group
    fn alu(&mut self, operand: u8, op: AluOp) {
        let xyq_val = match op {
            AluOp::Add => {
                self.update_flags_add(self.reg.a, operand, 0);
                self.reg.a += operand;
                self.reg.a
            }
            AluOp::Adc => {
                let cy = self.reg.f.contains(Flags::C) as u8;
                self.update_flags_add(self.reg.a, operand, cy);
                self.reg.a += operand + cy;
                self.reg.a
            }
            AluOp::Sub => {
                self.update_flags_sub(self.reg.a, operand, 0);
                self.reg.a -= operand;
                self.reg.a
            }
            AluOp::Sbc => {
                let cy = self.reg.f.contains(Flags::C) as u8;
                self.update_flags_sub(self.reg.a, operand, cy);
                self.reg.a -= operand + cy;
                self.reg.a
            }
            AluOp::And => {
                let res = self.reg.a & operand;
                self.update_flags_and(res);
                self.reg.a = res;
                self.reg.a
            }
            AluOp::Or => {
                let res = self.reg.a | operand;
                self.update_flags_or(res);
                self.reg.a = res;
                self.reg.a
            }
            AluOp::Xor => {
                let res = self.reg.a ^ operand;
                self.update_flags_or(res);
                self.reg.a = res;
                self.reg.a
            }
            AluOp::Cp => {
                self.update_flags_cmp(self.reg.a, operand);
                operand
            }
        };

        self.update_x_y_q(xyq_val);
    }

    fn alu_r(&mut self, bus: &mut B, src: Reg, op: AluOp) {
        let src_val = self.reg.get(src);
        self.alu(src_val, op);
        self.end_instruction(bus, true);
    }

    fn alu_n(&mut self, bus: &mut B, op: AluOp) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                let data = self.mem_rd_t3(bus);
                self.alu(data, op);
                self.end_instruction(bus, true);
            }
            _ => {}
        }
    }

    fn alu_hli(&mut self, bus: &mut B, op: AluOp) {
        match self.tcycle {
            5 => {
                let addr = self.reg.get_pair(RegPair::HL);
                self.mem_rd_t1(bus, addr);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                let data = self.mem_rd_t3(bus);
                self.alu(data, op);
                self.end_instruction(bus, true);
            }
            _ => {}
        }
    }

    fn inc_dec(&mut self, val: u8, inc: bool) -> u8 {
        let res = if inc {
            self.update_flags_inc(val);
            val + 1
        } else {
            self.update_flags_dec(val);
            val - 1
        };

        self.update_x_y_q(res);
        res
    }

    fn inc_dec_r(&mut self, bus: &mut B, dst: Reg, inc: bool) {
        let r = self.reg.get(dst);
        let res = self.inc_dec(r, inc);
        self.reg.set(dst, res);
        self.end_instruction(bus, true);
    }

    fn inc_dec_hli(&mut self, bus: &mut B, inc: bool) {
        match self.tcycle {
            5 => {
                let addr = self.reg.get_pair(RegPair::HL);
                self.mem_rd_t1(bus, addr);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                let res = self.inc_dec(self.reg.tmp[0], inc);
                self.reg.tmp[0] = res;
            }
            9 => {
                let addr = self.reg.get_pair(RegPair::HL);
                self.mem_wr_t1(bus, addr);
            }
            10 => {
                self.mem_wr_t2(bus, self.reg.tmp[0]);
            }
            11 => {
                self.mem_wr_t3(bus);
                self.end_instruction(bus, true);
            }
            _ => {}
        }
    }

    // General-Purpose Arithmetic and Logic Group
    fn daa(&mut self, bus: &mut B) {
        let (nib_high, nib_low) = (self.reg.a >> 4, self.reg.a & 0xF);

        let mut val = 0x00;
        if self.reg.f.contains(Flags::H) || (nib_low > 9) {
            val += 0x06;
        }
        if self.reg.f.contains(Flags::C) || (nib_high > 9) || (nib_high >= 9 && nib_low > 9) {
            val += 0x60;
            self.reg.f.insert(Flags::C);
        }

        if self.reg.f.contains(Flags::N) {
            self.update_flag_h_sub(self.reg.a, val, 0);
            self.reg.a -= val;
        } else {
            self.update_flag_h_add(self.reg.a, val, 0);
            self.reg.a += val;
        };

        self.update_flags_zps(self.reg.a);
        self.update_x_y_q(self.reg.a);
        self.end_instruction(bus, true);
    }

    fn cpl(&mut self, bus: &mut B) {
        self.reg.a = !self.reg.a;
        self.reg.f.insert(Flags::H | Flags::N);
        self.update_x_y_q(self.reg.a);
        self.end_instruction(bus, true);
    }

    fn ccf(&mut self, bus: &mut B) {
        self.reg.f.set(Flags::H, self.reg.f.contains(Flags::C));
        self.reg.f.toggle(Flags::C);
        self.reg.f.remove(Flags::N);
        self.update_x_y_q_alt(self.reg.a);
        self.end_instruction(bus, true);
    }

    fn scf(&mut self, bus: &mut B) {
        self.reg.f.insert(Flags::C);
        self.reg.f.remove(Flags::H);
        self.reg.f.remove(Flags::N);
        self.update_x_y_q_alt(self.reg.a);
        self.end_instruction(bus, true);
    }

    // CPU Control Group
    fn nop(&mut self, bus: &mut B) {
        self.end_instruction(bus, false);
    }

    fn halt(&mut self, bus: &mut B) {
        self.halt = true;
        self.end_instruction(bus, false);
    }

    fn ei(&mut self, bus: &mut B, val: bool) {
        self.int.ei = val;
        self.int.iff1 = val;
        self.int.iff2 = val;
        self.end_instruction(bus, false);
    }

    // 16-Bit Arithmetic Group
    fn add_hl_rr(&mut self, bus: &mut B, src: RegPair) {
        // No externally observable state is changed during earlier cycles
        // So for simplicity peform all work on last cycle
        if self.tcycle == 11 {
            // Perform op
            let dst_val = self.reg.get_pair(RegPair::HL);
            let src_val = self.reg.get_pair(src);
            let res = dst_val + src_val;
            self.reg.set_pair(RegPair::HL, res);

            // Update flags
            self.reg.f.remove(Flags::N);
            let h = (dst_val & 0xFFF) + (src_val & 0xFFF) > 0xFFF;
            self.reg.f.set(Flags::H, h);
            let c = dst_val as u32 + src_val as u32 > 0xFFFF;
            self.reg.f.set(Flags::C, c);
            self.update_x_y_q((res >> 8) as u8);

            self.reg.wz = dst_val + 1;
            self.end_instruction(bus, true);
        }
    }

    fn inc_dec_rr(&mut self, bus: &mut B, dst: RegPair, inc: bool) {
        // No externally observable state is changed during earlier cycles
        // So for simplicity peform all work on last cycle
        if self.tcycle == 6 {
            let dst_val = self.reg.get_pair(dst);
            let res = if inc { dst_val + 1 } else { dst_val - 1 };
            self.reg.set_pair(dst, res);
            self.end_instruction(bus, false);
        }
    }

    // Rotate and Shift Group
    fn ra(&mut self, bus: &mut B, left: bool, through_carry: bool) {
        let c = self.reg.f.contains(Flags::C) as u8;
        let (out_bit, c) = if left {
            let out_bit = self.reg.a >> 7;
            self.reg.a <<= 1;
            (out_bit, c)
        } else {
            let out_bit = self.reg.a << 7;
            self.reg.a >>= 1;
            (out_bit, c << 7)
        };

        if through_carry {
            self.reg.a |= c;
        } else {
            self.reg.a |= out_bit;
        }

        self.reg.f.set(Flags::C, out_bit != 0);
        self.reg.f.remove(Flags::H | Flags::N);
        self.update_x_y_q(self.reg.a);
        self.end_instruction(bus, true);
    }

    // Jump Group
    fn jp_nn(&mut self, bus: &mut B, cond: bool) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.mem_rd_t1_imm(bus);
            }
            9 => {
                self.mem_rd_t2(bus);
            }
            10 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
                let addr = u16::from_le_bytes(self.reg.tmp);
                if cond {
                    self.reg.pc = addr;
                }

                self.reg.wz = addr;
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn jr_end(&mut self, bus: &mut B) {
        self.reg.pc += (self.reg.tmp[0] as i8) as u16;
        self.reg.wz = self.reg.pc;
        self.end_instruction(bus, false);
    }

    fn jr_dis(&mut self, bus: &mut B, cond: bool) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
                if !cond {
                    self.end_instruction(bus, false);
                }
            }
            // 5 cycles here with no externally observable state change...
            12 => {
                self.jr_end(bus);
            }
            _ => {}
        }
    }

    fn jp_hli(&mut self, bus: &mut B) {
        self.reg.pc = self.reg.get_pair(RegPair::HL);
        self.end_instruction(bus, false);
    }

    fn djnz_dis(&mut self, bus: &mut B) {
        match self.tcycle {
            6 => {
                self.mem_rd_t1_imm(bus);
            }
            7 => {
                self.mem_rd_t2(bus);
            }
            8 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
                self.reg.b -= 1;
                if self.reg.b == 0 {
                    self.end_instruction(bus, false);
                }
            }
            // 5 cycles here with no externally observable state change...
            13 => {
                self.jr_end(bus);
            }
            _ => {}
        }
    }

    // Call and Return Group
    fn call(&mut self, bus: &mut B, cond: bool) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => {
                self.mem_rd_t2(bus);
            }
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.mem_rd_t1_imm(bus);
            }
            9 => {
                self.mem_rd_t2(bus);
            }
            10 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
                self.reg.wz = u16::from_le_bytes(self.reg.tmp);
                if !cond {
                    self.end_instruction(bus, false);
                }
            }
            11 => {}
            12 => {
                self.mem_wr_t1_push(bus);
            }
            13 => {
                self.mem_wr_t2(bus, (self.reg.pc >> 8) as u8);
            }
            14 => {
                self.mem_wr_t3(bus);
            }
            15 => {
                self.mem_wr_t1_push(bus);
            }
            16 => {
                self.mem_wr_t2(bus, self.reg.pc as u8);
            }
            17 => {
                self.mem_wr_t3(bus);
                self.reg.pc = u16::from_le_bytes(self.reg.tmp);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ret(&mut self, bus: &mut B, start_cycle: TCycles) {
        match self.tcycle - start_cycle {
            0 => {
                self.mem_rd_t1_pop(bus);
            }
            1 => {
                self.mem_rd_t2(bus);
            }
            2 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            3 => {
                self.mem_rd_t1_pop(bus);
            }
            4 => {
                self.mem_rd_t2(bus);
            }
            5 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
                self.reg.pc = u16::from_le_bytes(self.reg.tmp);
                self.reg.wz = self.reg.pc;
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn ret_cc(&mut self, bus: &mut B, cond: bool) {
        if !cond && self.tcycle == 5 {
            self.end_instruction(bus, false);
        } else if self.tcycle >= 6 {
            self.ret(bus, 6);
        }
    }

    fn rst(&mut self, bus: &mut B, rst: u16) {
        match self.tcycle {
            5 => {}
            6 => {
                self.mem_wr_t1_push(bus);
            }
            7 => {
                self.mem_wr_t2(bus, (self.reg.pc >> 8) as u8);
            }
            8 => {
                self.mem_wr_t3(bus);
            }
            9 => {
                self.mem_wr_t1_push(bus);
            }
            10 => {
                self.mem_wr_t2(bus, self.reg.pc as u8);
            }
            11 => {
                self.mem_wr_t3(bus);
                self.reg.pc = rst;
                self.reg.wz = rst;
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    // Input and Output Group
    fn in_a_n(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => self.mem_rd_t2(bus),
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.io_rd_t1(bus, self.reg.tmp[0]);
                self.reg.wz = bus.addr() + 1;
            }
            9 => self.io_rd_t2(bus),
            10 => self.io_rd_t3(bus),
            11 => {
                self.reg.a = self.io_rd_t4(bus);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    fn out_n_a(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1_imm(bus);
            }
            6 => self.mem_rd_t2(bus),
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                let port = self.reg.tmp[0];
                self.io_wr_t1(bus, port);
                self.reg.wz = u16::from_be_bytes([self.reg.a, port + 1]);
            }
            9 => self.io_wr_t2(bus),
            10 => self.io_wr_t3(bus, self.reg.a),
            11 => {
                self.io_wr_t4(bus);
                self.end_instruction(bus, false);
            }
            _ => {}
        }
    }

    pub(crate) fn end_instruction(&mut self, bus: &mut B, flags_updated: bool) {
        self.tcycle = 0;
        if !flags_updated {
            self.reg.q = 0;
        }

        if bus.nmi() {
            self.handle_nmi(bus);
        } else if !self.int.ei && self.int.iff1 && bus.int() {
            self.handle_int(bus);
        }
    }
}
