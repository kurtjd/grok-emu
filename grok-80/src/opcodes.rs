use crate::{BusHandlerZ80, Cpu, Reg, RegPair, opcodes_cb::OpcodeCB};

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
    RST_0,
    RET_Z,
    RET,
    JP_Z_NN,
    PREFIX_CB,
    CALL_Z_NN,
    CALL_NN,
    ADC_A_N,
    RST_8,
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
    SBD_A_N,
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
                self.end_instruction(bus);
            }

            Opcode::IN_A_N => match self.tcycle {
                5 => {
                    self.mem_rd_t1(bus, self.reg.pc);
                    self.reg.pc += 1;
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
                    self.end_instruction(bus);
                }
                _ => {}
            },
            Opcode::OUT_N_A => match self.tcycle {
                5 => {
                    self.mem_rd_t1(bus, self.reg.pc);
                    self.reg.pc += 1;
                }
                6 => self.mem_rd_t2(bus),
                7 => {
                    let data = self.mem_rd_t3(bus);
                    self.reg.tmp[0] = data;
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
                    self.end_instruction(bus);
                }
                _ => {}
            },
            Opcode::EI => {
                self.int.ei = true;
                self.int.iff1 = true;
                self.int.iff2 = true;
            }
            _ => todo!("Opcode: {:?}", opcode),
        }
    }

    // 8-Bit Load Group Functions
    fn ld_r_r(&mut self, bus: &mut B, dst: Reg, src: Reg) {
        self.reg.set(dst, self.reg.get(src));
        self.end_instruction(bus);
    }

    fn ld_r_n(&mut self, bus: &mut B, dst: Reg) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1(bus, self.reg.pc);
                self.reg.pc += 1;
            }
            6 => self.mem_rd_t2(bus),
            7 => {
                let data = self.mem_rd_t3(bus);
                self.reg.set(dst, data);
                self.end_instruction(bus);
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
            6 => self.mem_rd_t2(bus),
            7 => {
                let data = self.mem_rd_t3(bus);
                self.reg.set(dst, data);

                // Special case for `LD A, (BC)` and `LD A, (DE)`
                if dst == Reg::A && (src == RegPair::BC || src == RegPair::DE) {
                    self.reg.wz = self.reg.get_pair(src) + 1;
                }

                self.end_instruction(bus);
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
            6 => self.mem_wr_t2(bus, self.reg.get(src)),
            7 => {
                self.mem_wr_t3(bus);

                // Special case for `LD (BC), A` and `LD (DE), A`
                if src == Reg::A && (dst == RegPair::BC || dst == RegPair::DE) {
                    self.reg.wz =
                        u16::from_be_bytes([self.reg.a, (self.reg.get_pair(dst) + 1) as u8]);
                }

                self.end_instruction(bus);
            }
            _ => {}
        }
    }

    fn ld_hli_n(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1(bus, self.reg.pc);
                self.reg.pc += 1;
            }
            6 => self.mem_rd_t2(bus),
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                let addr = self.reg.get_pair(RegPair::HL);
                self.mem_wr_t1(bus, addr);
            }
            9 => self.mem_wr_t2(bus, self.reg.tmp[0]),
            10 => {
                self.mem_wr_t3(bus);
                self.end_instruction(bus);
            }
            _ => {}
        }
    }

    fn ld_a_nni(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1(bus, self.reg.pc);
                self.reg.pc += 1;
            }
            6 => self.mem_rd_t2(bus),
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.mem_rd_t1(bus, self.reg.pc);
                self.reg.pc += 1;
            }
            9 => self.mem_rd_t2(bus),
            10 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
            }
            11 => {
                let addr = u16::from_le_bytes(self.reg.tmp);
                self.mem_rd_t1(bus, addr);
                self.reg.wz = addr + 1;
            }
            12 => self.mem_rd_t2(bus),
            13 => {
                self.reg.a = self.mem_rd_t3(bus);
                self.end_instruction(bus);
            }
            _ => {}
        }
    }

    fn ld_nni_a(&mut self, bus: &mut B) {
        match self.tcycle {
            5 => {
                self.mem_rd_t1(bus, self.reg.pc);
                self.reg.pc += 1;
            }
            6 => self.mem_rd_t2(bus),
            7 => {
                self.reg.tmp[0] = self.mem_rd_t3(bus);
            }
            8 => {
                self.mem_rd_t1(bus, self.reg.pc);
                self.reg.pc += 1;
            }
            9 => self.mem_rd_t2(bus),
            10 => {
                self.reg.tmp[1] = self.mem_rd_t3(bus);
            }
            11 => {
                let addr = u16::from_le_bytes(self.reg.tmp);
                self.mem_wr_t1(bus, addr);
                self.reg.wz = u16::from_be_bytes([self.reg.a, (addr + 1) as u8]);
            }
            12 => self.mem_wr_t2(bus, self.reg.a),
            13 => {
                self.mem_wr_t3(bus);
                self.end_instruction(bus);
            }
            _ => {}
        }
    }
}
