use crate::{BusHandlerZ80, Cpu, opcodes_cb::OpcodeCB};
use std::mem::swap;

#[derive(Copy, Clone, Debug, num_enum::FromPrimitive)]
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
    EX_AF_AF,
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
    PREFIX_CB,
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
    EXX,
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

impl<B: BusHandlerZ80> Cpu<B> {
    pub(crate) fn execute(&mut self, opcode: Opcode, bus: &mut B) {
        match opcode {
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
                swap(&mut self.reg.gpr.b, &mut self.reg.gpr_alt.b);
                swap(&mut self.reg.gpr.c, &mut self.reg.gpr_alt.c);
                swap(&mut self.reg.gpr.d, &mut self.reg.gpr_alt.d);
                swap(&mut self.reg.gpr.e, &mut self.reg.gpr_alt.e);
                swap(&mut self.reg.gpr.h, &mut self.reg.gpr_alt.h);
                swap(&mut self.reg.gpr.l, &mut self.reg.gpr_alt.l);
                self.end_instruction(bus);
            }
            Opcode::MOV_B_C => {
                self.reg.gpr.b = self.reg.gpr.c;
                self.end_instruction(bus);
            }
            Opcode::MOV_B_M => match self.tcycle {
                5 => {
                    let addr = self.reg_pair(self.reg.gpr.h, self.reg.gpr.l);
                    self.mem_rd_t1(bus, addr);
                }
                6 => self.mem_rd_t2(bus),
                7 => {
                    let data = self.mem_rd_t3(bus);
                    self.reg.gpr.b = data;
                    self.end_instruction(bus);
                }
                _ => {}
            },
            Opcode::MOV_M_B => match self.tcycle {
                5 => {
                    let addr = self.reg_pair(self.reg.gpr.h, self.reg.gpr.l);
                    self.mem_wr_t1(bus, addr);
                }
                6 => self.mem_wr_t2(bus, self.reg.gpr.b),
                7 => {
                    self.mem_wr_t3(bus);
                    self.end_instruction(bus);
                }
                _ => {}
            },
            Opcode::IN => match self.tcycle {
                5 => {
                    self.mem_rd_t1(bus, self.reg.spr.pc);
                    self.reg.spr.pc += 1;
                }
                6 => self.mem_rd_t2(bus),
                7 => {
                    let data = self.mem_rd_t3(bus);
                    self.reg.wpr.z = data;
                }
                8 => {
                    self.io_rd_t1(bus, self.reg.wpr.z);
                    [self.reg.wpr.w, self.reg.wpr.z] = (bus.addr() + 1).to_be_bytes();
                }
                9 => self.io_rd_t2(bus),
                10 => self.io_rd_t3(bus),
                11 => {
                    self.reg.gpr.a = self.io_rd_t4(bus);
                    self.end_instruction(bus);
                }
                _ => {}
            },
            Opcode::OUT => match self.tcycle {
                5 => {
                    self.mem_rd_t1(bus, self.reg.spr.pc);
                    self.reg.spr.pc += 1;
                }
                6 => self.mem_rd_t2(bus),
                7 => {
                    let data = self.mem_rd_t3(bus);
                    self.reg.wpr.z = data;
                }
                8 => {
                    let port = self.reg.wpr.z;
                    self.io_wr_t1(bus, port);
                    self.reg.wpr.w = self.reg.gpr.a;
                    self.reg.wpr.z = port + 1;
                }
                9 => self.io_wr_t2(bus),
                10 => self.io_wr_t3(bus, self.reg.gpr.a),
                11 => {
                    self.io_wr_t4(bus);
                    self.end_instruction(bus);
                }
                _ => {}
            },
            _ => todo!("Opcode: {:?}", opcode),
        }
    }
}
