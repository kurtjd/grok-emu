use num_enum::FromPrimitive;

use crate::TCycles;

const MCYCLES_MAX: usize = 5;

#[cfg(feature = "sm83")]
const T_PER_M: TCycles = 4;

pub struct OpcodeInfo {
    pub name: &'static str,
    pub len: u8,
    pub t_per_m: [Option<TCycles>; MCYCLES_MAX],
}

#[derive(Copy, Clone, Debug, FromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Opcode {
    NOP,
    LXI_B,
    STAX_B,
    INX_B,
    INR_B,
    DCR_B,
    MVI_B,
    RLC,
    UNDEF_1,
    DAD_B,
    LDAX_B,
    DCX_B,
    INR_C,
    DCR_C,
    MVI_C,
    RRC,
    UNDEF_2,
    LXI_D,
    STAX_D,
    INX_D,
    INR_D,
    DCR_D,
    MVI_D,
    RAL,
    UNDEF_3,
    DAD_D,
    LDAX_D,
    DCX_D,
    INR_E,
    DCR_E,
    MVI_E,
    RAR,
    UNDEF_4,
    LXI_H,
    SHLD,
    INX_H,
    INR_H,
    DCR_H,
    MVI_H,
    DAA,
    UNDEF_5,
    DAD_H,
    LHLD,
    DCX_H,
    INR_L,
    DCR_L,
    MVI_L,
    CMA,
    UNDEF_6,
    LXI_SP,
    STA,
    INX_SP,
    INR_M,
    DCR_M,
    MVI_M,
    STC,
    UNDEF_7,
    DAD_SP,
    LDA,
    DCX_SP,
    INR_A,
    DCR_A,
    MVI_A,
    CMC,
    MOV_B_B,
    MOV_B_C,
    MOV_B_D,
    MOV_B_E,
    MOV_B_H,
    MOV_B_L,
    MOV_B_M,
    MOV_B_A,
    MOV_C_B,
    MOV_C_C,
    MOV_C_D,
    MOV_C_E,
    MOV_C_H,
    MOV_C_L,
    MOV_C_M,
    MOV_C_A,
    MOV_D_B,
    MOV_D_C,
    MOV_D_D,
    MOV_D_E,
    MOV_D_H,
    MOV_D_L,
    MOV_D_M,
    MOV_D_A,
    MOV_E_B,
    MOV_E_C,
    MOV_E_D,
    MOV_E_E,
    MOV_E_H,
    MOV_E_L,
    MOV_E_M,
    MOV_E_A,
    MOV_H_B,
    MOV_H_C,
    MOV_H_D,
    MOV_H_E,
    MOV_H_H,
    MOV_H_L,
    MOV_H_M,
    MOV_H_A,
    MOV_L_B,
    MOV_L_C,
    MOV_L_D,
    MOV_L_E,
    MOV_L_H,
    MOV_L_L,
    MOV_L_M,
    MOV_L_A,
    MOV_M_B,
    MOV_M_C,
    MOV_M_D,
    MOV_M_E,
    MOV_M_H,
    MOV_M_L,
    HLT,
    MOV_M_A,
    MOV_A_B,
    MOV_A_C,
    MOV_A_D,
    MOV_A_E,
    MOV_A_H,
    MOV_A_L,
    MOV_A_M,
    MOV_A_A,
    ADD_B,
    ADD_C,
    ADD_D,
    ADD_E,
    ADD_H,
    ADD_L,
    ADD_M,
    ADD_A,
    ADC_B,
    ADC_C,
    ADC_D,
    ADC_E,
    ADC_H,
    ADC_L,
    ADC_M,
    ADC_A,
    SUB_B,
    SUB_C,
    SUB_D,
    SUB_E,
    SUB_H,
    SUB_L,
    SUB_M,
    SUB_A,
    SBB_B,
    SBB_C,
    SBB_D,
    SBB_E,
    SBB_H,
    SBB_L,
    SBB_M,
    SBB_A,
    ANA_B,
    ANA_C,
    ANA_D,
    ANA_E,
    ANA_H,
    ANA_L,
    ANA_M,
    ANA_A,
    XRA_B,
    XRA_C,
    XRA_D,
    XRA_E,
    XRA_H,
    XRA_L,
    XRA_M,
    XRA_A,
    ORA_B,
    ORA_C,
    ORA_D,
    ORA_E,
    ORA_H,
    ORA_L,
    ORA_M,
    ORA_A,
    CMP_B,
    CMP_C,
    CMP_D,
    CMP_E,
    CMP_H,
    CMP_L,
    CMP_M,
    CMP_A,
    RNZ,
    POP_B,
    JNZ,
    JMP,
    CNZ,
    PUSH_B,
    ADI,
    RST_0,
    RZ,
    RET,
    JZ,
    UNDEF_8,
    CZ,
    CALL,
    ACI,
    RST_1,
    RNC,
    POP_D,
    JNC,
    OUT,
    CNC,
    PUSH_D,
    SUI,
    RST_2,
    RC,
    UNDEF_9,
    JC,
    IN,
    CC,
    UNDEF_10,
    SBI,
    RST_3,
    RPO,
    POP_H,
    JPO,
    XTHL,
    CPO,
    PUSH_H,
    ANI,
    RST_4,
    RPE,
    PCHL,
    JPE,
    XCHG,
    CPE,
    UNDEF_11,
    XRI,
    RST_5,
    RP,
    POP_PSW,
    JP,
    DI,
    CP,
    PUSH_PSW,
    ORI,
    RST_6,
    RM,
    SPHL,
    JM,
    EI,
    CM,
    UNDEF_12,
    CPI,
    RST_7,
}

impl Opcode {
    pub const fn info(&self) -> OpcodeInfo {
        match self {
            // Data Transfer Group
            Opcode::MOV_A_A => OpcodeInfo {
                name: "MOV A,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_A_B => OpcodeInfo {
                name: "MOV A,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_A_C => OpcodeInfo {
                name: "MOV A,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_A_D => OpcodeInfo {
                name: "MOV A,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_A_E => OpcodeInfo {
                name: "MOV A,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_A_H => OpcodeInfo {
                name: "MOV A,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_A_L => OpcodeInfo {
                name: "MOV A,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_A => OpcodeInfo {
                name: "MOV B,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_B => OpcodeInfo {
                name: "MOV B,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_C => OpcodeInfo {
                name: "MOV B,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_D => OpcodeInfo {
                name: "MOV B,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_E => OpcodeInfo {
                name: "MOV B,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_H => OpcodeInfo {
                name: "MOV B,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_B_L => OpcodeInfo {
                name: "MOV B,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_A => OpcodeInfo {
                name: "MOV C,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_B => OpcodeInfo {
                name: "MOV C,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_C => OpcodeInfo {
                name: "MOV C,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_D => OpcodeInfo {
                name: "MOV C,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_E => OpcodeInfo {
                name: "MOV C,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_H => OpcodeInfo {
                name: "MOV C,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_C_L => OpcodeInfo {
                name: "MOV C,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_A => OpcodeInfo {
                name: "MOV D,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_B => OpcodeInfo {
                name: "MOV D,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_C => OpcodeInfo {
                name: "MOV D,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_D => OpcodeInfo {
                name: "MOV D,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_E => OpcodeInfo {
                name: "MOV D,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_H => OpcodeInfo {
                name: "MOV D,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_D_L => OpcodeInfo {
                name: "MOV D,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_A => OpcodeInfo {
                name: "MOV E,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_B => OpcodeInfo {
                name: "MOV E,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_C => OpcodeInfo {
                name: "MOV E,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_D => OpcodeInfo {
                name: "MOV E,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_E => OpcodeInfo {
                name: "MOV E,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_H => OpcodeInfo {
                name: "MOV E,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_E_L => OpcodeInfo {
                name: "MOV E,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_A => OpcodeInfo {
                name: "MOV H,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_B => OpcodeInfo {
                name: "MOV H,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_C => OpcodeInfo {
                name: "MOV H,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_D => OpcodeInfo {
                name: "MOV H,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_E => OpcodeInfo {
                name: "MOV H,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_H => OpcodeInfo {
                name: "MOV H,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_H_L => OpcodeInfo {
                name: "MOV H,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_A => OpcodeInfo {
                name: "MOV L,A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_B => OpcodeInfo {
                name: "MOV L,B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_C => OpcodeInfo {
                name: "MOV L,C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_D => OpcodeInfo {
                name: "MOV L,D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_E => OpcodeInfo {
                name: "MOV L,E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_H => OpcodeInfo {
                name: "MOV L,H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::MOV_L_L => OpcodeInfo {
                name: "MOV L,L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },

            Opcode::MOV_A_M => OpcodeInfo {
                name: "MOV A,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_B_M => OpcodeInfo {
                name: "MOV B,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_C_M => OpcodeInfo {
                name: "MOV C,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_D_M => OpcodeInfo {
                name: "MOV D,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_E_M => OpcodeInfo {
                name: "MOV E,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_H_M => OpcodeInfo {
                name: "MOV H,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_L_M => OpcodeInfo {
                name: "MOV L,M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::MOV_M_A => OpcodeInfo {
                name: "MOV M,A",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_M_B => OpcodeInfo {
                name: "MOV M,B",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_M_C => OpcodeInfo {
                name: "MOV M,C",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_M_D => OpcodeInfo {
                name: "MOV M,D",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_M_E => OpcodeInfo {
                name: "MOV M,E",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_M_H => OpcodeInfo {
                name: "MOV M,H",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MOV_M_L => OpcodeInfo {
                name: "MOV M,L",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::MVI_A => OpcodeInfo {
                name: "MVI A,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_B => OpcodeInfo {
                name: "MVI B,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_C => OpcodeInfo {
                name: "MVI C,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_D => OpcodeInfo {
                name: "MVI D,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_E => OpcodeInfo {
                name: "MVI E,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_H => OpcodeInfo {
                name: "MVI H,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_L => OpcodeInfo {
                name: "MVI L,",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::MVI_M => OpcodeInfo {
                name: "MVI M,",
                len: 2,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::LXI_B => OpcodeInfo {
                name: "LXI B,",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::LXI_D => OpcodeInfo {
                name: "LXI D,",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::LXI_H => OpcodeInfo {
                name: "LXI H,",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::LXI_SP => OpcodeInfo {
                name: "LXI SP,",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::LDA => OpcodeInfo {
                name: "LDA ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), Some(3), None],
            },
            Opcode::STA => OpcodeInfo {
                name: "STA ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), Some(3), None],
            },
            Opcode::LHLD => OpcodeInfo {
                name: "LHLD ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), Some(3), Some(3)],
            },
            Opcode::SHLD => OpcodeInfo {
                name: "SHLD ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), Some(3), Some(3)],
            },

            Opcode::LDAX_B => OpcodeInfo {
                name: "LDAX B",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::LDAX_D => OpcodeInfo {
                name: "LDAX D",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::STAX_B => OpcodeInfo {
                name: "STAX B",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::STAX_D => OpcodeInfo {
                name: "STAX D",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::XCHG => OpcodeInfo {
                name: "XCHG",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },

            Opcode::ADD_A => OpcodeInfo {
                name: "ADD A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_B => OpcodeInfo {
                name: "ADD B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_C => OpcodeInfo {
                name: "ADD C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_D => OpcodeInfo {
                name: "ADD D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_E => OpcodeInfo {
                name: "ADD E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_H => OpcodeInfo {
                name: "ADD H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_L => OpcodeInfo {
                name: "ADD L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADD_M => OpcodeInfo {
                name: "ADD M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::ADI => OpcodeInfo {
                name: "ADI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::ADC_A => OpcodeInfo {
                name: "ADC A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_B => OpcodeInfo {
                name: "ADC B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_C => OpcodeInfo {
                name: "ADC C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_D => OpcodeInfo {
                name: "ADC D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_E => OpcodeInfo {
                name: "ADC E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_H => OpcodeInfo {
                name: "ADC H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_L => OpcodeInfo {
                name: "ADC L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ADC_M => OpcodeInfo {
                name: "ADC M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::ACI => OpcodeInfo {
                name: "ACI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::SUB_A => OpcodeInfo {
                name: "SUB A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_B => OpcodeInfo {
                name: "SUB B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_C => OpcodeInfo {
                name: "SUB C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_D => OpcodeInfo {
                name: "SUB D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_E => OpcodeInfo {
                name: "SUB E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_H => OpcodeInfo {
                name: "SUB H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_L => OpcodeInfo {
                name: "SUB L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SUB_M => OpcodeInfo {
                name: "SUB M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::SUI => OpcodeInfo {
                name: "SUI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::SBB_A => OpcodeInfo {
                name: "SBB A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_B => OpcodeInfo {
                name: "SBB B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_C => OpcodeInfo {
                name: "SBB C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_D => OpcodeInfo {
                name: "SBB D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_E => OpcodeInfo {
                name: "SBB E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_H => OpcodeInfo {
                name: "SBB H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_L => OpcodeInfo {
                name: "SBB L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::SBB_M => OpcodeInfo {
                name: "SBB M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::SBI => OpcodeInfo {
                name: "SBI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::INR_A => OpcodeInfo {
                name: "INR A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_B => OpcodeInfo {
                name: "INR B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_C => OpcodeInfo {
                name: "INR C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_D => OpcodeInfo {
                name: "INR D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_E => OpcodeInfo {
                name: "INR E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_H => OpcodeInfo {
                name: "INR H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_L => OpcodeInfo {
                name: "INR L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::INR_M => OpcodeInfo {
                name: "INR M",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::DCR_A => OpcodeInfo {
                name: "DCR A",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_B => OpcodeInfo {
                name: "DCR B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_C => OpcodeInfo {
                name: "DCR C",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_D => OpcodeInfo {
                name: "DCR D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_E => OpcodeInfo {
                name: "DCR E",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_H => OpcodeInfo {
                name: "DCR H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_L => OpcodeInfo {
                name: "DCR L",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DCR_M => OpcodeInfo {
                name: "DCR M",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::INX_B => OpcodeInfo {
                name: "INX B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::INX_D => OpcodeInfo {
                name: "INX D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::INX_H => OpcodeInfo {
                name: "INX H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::INX_SP => OpcodeInfo {
                name: "INX SP",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },

            Opcode::DCX_B => OpcodeInfo {
                name: "DCX B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::DCX_D => OpcodeInfo {
                name: "DCX D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::DCX_H => OpcodeInfo {
                name: "DCX H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::DCX_SP => OpcodeInfo {
                name: "DCX SP",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },

            Opcode::DAD_B => OpcodeInfo {
                name: "DAD B",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::DAD_D => OpcodeInfo {
                name: "DAD D",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::DAD_H => OpcodeInfo {
                name: "DAD H",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::DAD_SP => OpcodeInfo {
                name: "DAD SP",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::DAA => OpcodeInfo {
                name: "DAA",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },

            Opcode::ANA_A => OpcodeInfo {
                name: "ANA A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_B => OpcodeInfo {
                name: "ANA B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_C => OpcodeInfo {
                name: "ANA C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_D => OpcodeInfo {
                name: "ANA D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_E => OpcodeInfo {
                name: "ANA E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_H => OpcodeInfo {
                name: "ANA H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_L => OpcodeInfo {
                name: "ANA L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ANA_M => OpcodeInfo {
                name: "ANA M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::ANI => OpcodeInfo {
                name: "ANI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::XRA_A => OpcodeInfo {
                name: "XRA A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_B => OpcodeInfo {
                name: "XRA B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_C => OpcodeInfo {
                name: "XRA C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_D => OpcodeInfo {
                name: "XRA D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_E => OpcodeInfo {
                name: "XRA E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_H => OpcodeInfo {
                name: "XRA H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_L => OpcodeInfo {
                name: "XRA L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::XRA_M => OpcodeInfo {
                name: "XRA M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::XRI => OpcodeInfo {
                name: "XRI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::ORA_A => OpcodeInfo {
                name: "ORA A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_B => OpcodeInfo {
                name: "ORA B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_C => OpcodeInfo {
                name: "ORA C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_D => OpcodeInfo {
                name: "ORA D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_E => OpcodeInfo {
                name: "ORA E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_H => OpcodeInfo {
                name: "ORA H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_L => OpcodeInfo {
                name: "ORA L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::ORA_M => OpcodeInfo {
                name: "ORA M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::ORI => OpcodeInfo {
                name: "ORI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::CMP_A => OpcodeInfo {
                name: "CMP A",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_B => OpcodeInfo {
                name: "CMP B",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_C => OpcodeInfo {
                name: "CMP C",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_D => OpcodeInfo {
                name: "CMP D",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_E => OpcodeInfo {
                name: "CMP E",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_H => OpcodeInfo {
                name: "CMP H",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_L => OpcodeInfo {
                name: "CMP L",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMP_M => OpcodeInfo {
                name: "CMP M",
                len: 1,
                t_per_m: [Some(4), Some(3), None, None, None],
            },

            Opcode::CPI => OpcodeInfo {
                name: "CPI ",
                len: 2,
                t_per_m: [Some(4), Some(3), None, None, None],
            },
            Opcode::RLC => OpcodeInfo {
                name: "RLC",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::RRC => OpcodeInfo {
                name: "RRC",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::RAL => OpcodeInfo {
                name: "RAL",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::RAR => OpcodeInfo {
                name: "RAR",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMA => OpcodeInfo {
                name: "CMA",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::CMC => OpcodeInfo {
                name: "CMC",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::STC => OpcodeInfo {
                name: "STC",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::JMP => OpcodeInfo {
                name: "JMP ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::JNZ => OpcodeInfo {
                name: "JNZ ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JZ => OpcodeInfo {
                name: "JZ ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JNC => OpcodeInfo {
                name: "JNC ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JC => OpcodeInfo {
                name: "JC ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JPO => OpcodeInfo {
                name: "JPO ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JPE => OpcodeInfo {
                name: "JPE ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JP => OpcodeInfo {
                name: "JP ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::JM => OpcodeInfo {
                name: "JM ",
                len: 3,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::CALL => OpcodeInfo {
                name: "CALL ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(3), Some(3), Some(4)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(3), Some(3), Some(4), Some(4)],
            },

            Opcode::CNZ => OpcodeInfo {
                name: "CNZ ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CZ => OpcodeInfo {
                name: "CZ ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CNC => OpcodeInfo {
                name: "CNC ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CC => OpcodeInfo {
                name: "CC ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CPO => OpcodeInfo {
                name: "CPO ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CPE => OpcodeInfo {
                name: "CPE ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CP => OpcodeInfo {
                name: "CP ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },
            Opcode::CM => OpcodeInfo {
                name: "CM ",
                len: 3,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), Some(3), Some(3)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(5), Some(3), Some(3), Some(3)],
            },

            Opcode::RET => OpcodeInfo {
                name: "RET",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::RNZ => OpcodeInfo {
                name: "RNZ",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RZ => OpcodeInfo {
                name: "RZ",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RNC => OpcodeInfo {
                name: "RNC",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RC => OpcodeInfo {
                name: "RC",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RPO => OpcodeInfo {
                name: "RPO",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RPE => OpcodeInfo {
                name: "RPE",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RP => OpcodeInfo {
                name: "RP",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },
            Opcode::RM => OpcodeInfo {
                name: "RM",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), Some(3), Some(3), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), Some(3), Some(3), None, None],
            },

            Opcode::RST_0 => OpcodeInfo {
                name: "RST 0",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_1 => OpcodeInfo {
                name: "RST 1",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_2 => OpcodeInfo {
                name: "RST 2",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_3 => OpcodeInfo {
                name: "RST 3",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_4 => OpcodeInfo {
                name: "RST 4",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_5 => OpcodeInfo {
                name: "RST 5",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_6 => OpcodeInfo {
                name: "RST 6",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },
            Opcode::RST_7 => OpcodeInfo {
                name: "RST 7",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(4), None, None],
            },

            Opcode::PCHL => OpcodeInfo {
                name: "PCHL",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },

            Opcode::PUSH_B => OpcodeInfo {
                name: "PUSH B",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(5), None, None],
            },
            Opcode::PUSH_D => OpcodeInfo {
                name: "PUSH D",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(5), None, None],
            },
            Opcode::PUSH_H => OpcodeInfo {
                name: "PUSH H",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(5), None, None],
            },
            Opcode::PUSH_PSW => OpcodeInfo {
                name: "PUSH PSW",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(4), None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(4), Some(5), None, None],
            },

            Opcode::POP_B => OpcodeInfo {
                name: "POP B",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::POP_D => OpcodeInfo {
                name: "POP D",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::POP_H => OpcodeInfo {
                name: "POP H",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::POP_PSW => OpcodeInfo {
                name: "POP PSW",
                len: 1,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },

            Opcode::XTHL => OpcodeInfo {
                name: "XTHL",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(4), Some(3), Some(3), Some(4), Some(4)],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(4), Some(3), Some(3), Some(3), Some(3)],
            },
            Opcode::SPHL => OpcodeInfo {
                name: "SPHL",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(5), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(6), None, None, None, None],
            },
            Opcode::IN => OpcodeInfo {
                name: "IN ",
                len: 2,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::OUT => OpcodeInfo {
                name: "OUT ",
                len: 2,
                t_per_m: [Some(4), Some(3), Some(3), None, None],
            },
            Opcode::EI => OpcodeInfo {
                name: "EI",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::DI => OpcodeInfo {
                name: "DI",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::HLT => OpcodeInfo {
                name: "HLT",
                len: 1,
                #[cfg(feature = "i8080")]
                t_per_m: [Some(7), None, None, None, None],
                #[cfg(feature = "i8085")]
                t_per_m: [Some(5), None, None, None, None],
            },
            Opcode::NOP => OpcodeInfo {
                name: "NOP",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },

            Opcode::UNDEF_1 => OpcodeInfo {
                name: "UNDEF 1",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_2 => OpcodeInfo {
                name: "UNDEF 2",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_3 => OpcodeInfo {
                name: "UNDEF 3",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_4 => OpcodeInfo {
                name: "UNDEF 4",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_5 => OpcodeInfo {
                name: "UNDEF 5",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_6 => OpcodeInfo {
                name: "UNDEF 6",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_7 => OpcodeInfo {
                name: "UNDEF 7",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_8 => OpcodeInfo {
                name: "UNDEF 8",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_9 => OpcodeInfo {
                name: "UNDEF 9",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_10 => OpcodeInfo {
                name: "UNDEF 10",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_11 => OpcodeInfo {
                name: "UNDEF 11",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
            Opcode::UNDEF_12 => OpcodeInfo {
                name: "UNDEF 12",
                len: 1,
                t_per_m: [Some(4), None, None, None, None],
            },
        }
    }
}
