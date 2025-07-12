use crate::{BusHandler, Cpu, Flags, IffState, Register, TCycles};

const T_PER_M: TCycles = 4;

#[cfg(feature = "i8080")]
const ALU_TCYCLES: TCycles = 1;
#[cfg(not(feature = "i8080"))]
const ALU_TCYCLES: TCycles = 0;

// Mainly used by instructions that only push micro ops to the pipeline
const NO_TCYCLES: TCycles = 0;

// These represent shared micro_ops between multiple instructions
impl<T: BusHandler> Cpu<T> {
    fn fetch_operand(&mut self, idx: usize) -> TCycles {
        self.tmp[idx] = self.bus.mem_read(self.pc);
        self.pc += 1;
        T_PER_M
    }

    fn mov_r_m_m2(&mut self, dest: Register) -> TCycles {
        let addr = self.get_reg_pair(Register::H, Register::L);
        let val = self.bus.mem_read(addr);
        self.gpr[dest] = val;
        T_PER_M
    }

    fn mov_m_r_m2(&mut self, src: Register) -> TCycles {
        let addr = self.get_reg_pair(Register::H, Register::L);
        let val = self.gpr[src];
        self.bus.mem_write(addr, val);
        T_PER_M
    }

    fn mvi_r_m2(&mut self, dest: Register) -> TCycles {
        let imm = self.bus.mem_read(self.pc);
        self.pc += 1;
        self.gpr[dest] = imm;
        T_PER_M
    }

    fn lxi_r_m3(&mut self, dest1: Register, dest2: Register) -> TCycles {
        let t_cycles = self.fetch_operand(1);
        let val = u16::from_le_bytes(self.tmp);
        self.set_reg_pair(dest1, dest2, val);
        t_cycles
    }

    fn ldax_r_m2(&mut self, src1: Register, src2: Register) -> TCycles {
        let addr = self.get_reg_pair(src1, src2);
        self.gpr[Register::A] = self.bus.mem_read(addr);
        T_PER_M
    }

    fn stax_r_m2(&mut self, src1: Register, src2: Register) -> TCycles {
        let addr = self.get_reg_pair(src1, src2);
        let val = self.gpr[Register::A];
        self.bus.mem_write(addr, val);
        T_PER_M
    }

    // DAD_R takes 3 M-Cycles (1: Fetch, 2: Add low byte, 3: Add high byte)
    // However, for simplicity, we handle M-cycles 2 and 3 all in one M-cycle,
    // and then treat the 3rd M-cycle as a NOP.
    // This is fine because no memory is touched, so the outside world is none the wiser.
    fn dad_r_m2(&mut self, src1: Register, src2: Register) -> TCycles {
        let pair1 = self.get_reg_pair(Register::H, Register::L) as u32;
        let pair2 = self.get_reg_pair(src1, src2) as u32;
        let res = pair1 + pair2;
        self.flags.set(Flags::CY, res > 0xFFFF);
        self.set_reg_pair(Register::H, Register::L, res as u16);
        T_PER_M
    }

    #[cfg(feature = "i8080")]
    fn jcc_m2(&mut self, _flag: Flags, _cmp: bool) -> TCycles {
        self.fetch_operand(0)
    }

    // Most variants are smart and only take a 3rd M-cycle (fetch high byte) if branch taken
    #[cfg(not(feature = "i8080"))]
    fn jcc_m2(&mut self, flag: Flags, cmp: bool) -> TCycles {
        let t_cycles = self.fetch_operand(0);

        if self.flags.contains(flag) == cmp {
            // M3: Fetch high byte of target addr and jump
            self.pipeline.push_back(|cpu| {
                cpu.fetch_operand(1);
                let addr = u16::from_le_bytes(cpu.tmp);
                cpu.pc = addr;
                T_PER_M
            });
        } else {
            self.pc = self.pc.wrapping_add(1);
        }

        t_cycles
    }

    // Intel 8080 is dumb and always takes a 3rd M-cycle (fetch high byte) regardless if branch taken
    #[cfg(feature = "i8080")]
    fn jcc_m3(&mut self, flag: Flags, cmp: bool) -> TCycles {
        let t_cycles = self.fetch_operand(1);

        if self.flags.contains(flag) == cmp {
            let addr = u16::from_le_bytes(self.tmp);
            self.pc = addr;
        }

        t_cycles
    }

    fn call_m4(&mut self) -> TCycles {
        self.sp = self.sp.wrapping_sub(1);
        let val = (self.pc >> 8) as u8;
        self.bus.mem_write(self.sp, val);
        T_PER_M
    }

    fn call_m5(&mut self) -> TCycles {
        self.sp = self.sp.wrapping_sub(1);
        let val = self.pc as u8;
        self.bus.mem_write(self.sp, val);
        self.pc = u16::from_le_bytes(self.tmp);
        T_PER_M
    }

    #[cfg(feature = "i8080")]
    fn ccc_m2(&mut self, _flag: Flags, _cmp: bool) -> TCycles {
        self.fetch_operand(0)
    }

    // Most variants are smart and only take a 3rd M-cycle (fetch high byte) if branch taken
    #[cfg(not(feature = "i8080"))]
    fn ccc_m2(&mut self, flag: Flags, cmp: bool) -> TCycles {
        let t_cycles = self.fetch_operand(0);

        if self.flags.contains(flag) == cmp {
            // M3: Fetch high byte of target addr
            self.pipeline.push_back(|cpu| cpu.fetch_operand(1));

            // M4: Push high byte of return addr
            self.pipeline.push_back(|cpu| cpu.call_m4());

            // M5: Push low byte of return addr, then jump
            self.pipeline.push_back(|cpu| cpu.call_m5());
        } else {
            self.pc = self.pc.wrapping_add(1);
        }

        t_cycles
    }

    // Intel 8080 is dumb and always takes a 3rd M-cycle (fetch high byte) regardless if branch taken
    #[cfg(feature = "i8080")]
    fn ccc_m3(&mut self, flag: Flags, cmp: bool) -> TCycles {
        let t_cycles = self.fetch_operand(1);

        if self.flags.contains(flag) == cmp {
            // M4: Push high byte of return addr
            self.pipeline.push_back(|cpu| cpu.call_m4());

            // M5: Push low byte of return addr, then jump
            self.pipeline.push_back(|cpu| cpu.call_m5());
        }

        t_cycles
    }

    fn ret_m2(&mut self) -> TCycles {
        self.tmp[0] = self.bus.mem_read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        T_PER_M
    }

    fn ret_m3(&mut self) -> TCycles {
        self.tmp[1] = self.bus.mem_read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.pc = u16::from_le_bytes(self.tmp);
        T_PER_M
    }

    fn rst_n_m2(&mut self) -> TCycles {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.mem_write(self.sp, (self.pc >> 8) as u8);
        T_PER_M
    }

    fn rst_n_m3(&mut self, n: u16) -> TCycles {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.mem_write(self.sp, self.pc as u8);
        self.pc = n * 0x08;
        T_PER_M
    }

    fn push_r(&mut self, src: Register) -> TCycles {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.mem_write(self.sp, self.gpr[src]);
        T_PER_M
    }

    fn pop_r(&mut self, dest: Register) -> TCycles {
        self.gpr[dest] = self.bus.mem_read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        T_PER_M
    }
}

// These represent actual instructions
impl<T: BusHandler> Cpu<T> {
    pub(crate) fn mov_r_r(&mut self, dest: Register, src: Register) -> TCycles {
        self.gpr[dest] = self.gpr[src];
        ALU_TCYCLES
    }

    pub(crate) fn mov_a_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::A));

        NO_TCYCLES
    }

    pub(crate) fn mov_b_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::B));

        NO_TCYCLES
    }

    pub(crate) fn mov_c_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::C));

        NO_TCYCLES
    }

    pub(crate) fn mov_d_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::D));

        NO_TCYCLES
    }

    pub(crate) fn mov_e_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::E));

        NO_TCYCLES
    }

    pub(crate) fn mov_h_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::H));

        NO_TCYCLES
    }

    pub(crate) fn mov_l_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| cpu.mov_r_m_m2(Register::L));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_a(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::A));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_b(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::B));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_c(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::C));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_d(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::D));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_e(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::E));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_h(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::H));

        NO_TCYCLES
    }

    pub(crate) fn mov_m_l(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline.push_back(|cpu| cpu.mov_m_r_m2(Register::L));

        NO_TCYCLES
    }

    pub(crate) fn mvi_a(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::A));

        NO_TCYCLES
    }

    pub(crate) fn mvi_b(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::B));

        NO_TCYCLES
    }

    pub(crate) fn mvi_c(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::C));

        NO_TCYCLES
    }

    pub(crate) fn mvi_d(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::D));

        NO_TCYCLES
    }

    pub(crate) fn mvi_e(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::E));

        NO_TCYCLES
    }

    pub(crate) fn mvi_h(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::H));

        NO_TCYCLES
    }

    pub(crate) fn mvi_l(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.mvi_r_m2(Register::L));

        NO_TCYCLES
    }

    pub(crate) fn mvi_m(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Store value in memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            cpu.bus.mem_write(addr, cpu.tmp[0]);

            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn lxi_b(&mut self) -> TCycles {
        // M2: Fetch low byte of immediate value
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of immediate value
        self.pipeline
            .push_back(|cpu| cpu.lxi_r_m3(Register::B, Register::C));

        NO_TCYCLES
    }

    pub(crate) fn lxi_d(&mut self) -> TCycles {
        // M2: Fetch low byte of immediate value
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of immediate value
        self.pipeline
            .push_back(|cpu| cpu.lxi_r_m3(Register::D, Register::E));

        NO_TCYCLES
    }

    pub(crate) fn lxi_h(&mut self) -> TCycles {
        // M2: Fetch low byte of immediate value
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of immediate value
        self.pipeline
            .push_back(|cpu| cpu.lxi_r_m3(Register::H, Register::L));

        NO_TCYCLES
    }

    pub(crate) fn lxi_sp(&mut self) -> TCycles {
        // M2: Fetch low byte of immediate value
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(1);
            cpu.sp = u16::from_le_bytes(cpu.tmp);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn lda(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(1));

        // M4: Load A from addr
        self.pipeline.push_back(|cpu| {
            let addr = u16::from_le_bytes(cpu.tmp);
            let val = cpu.bus.mem_read(addr);
            cpu.gpr[Register::A] = val;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn sta(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(1));

        // M4: Store A at addr
        self.pipeline.push_back(|cpu| {
            let addr = u16::from_le_bytes(cpu.tmp);
            let val = cpu.gpr[Register::A];
            cpu.bus.mem_write(addr, val);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn lhld(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(1));

        // M4: Load L from addr
        self.pipeline.push_back(|cpu| {
            let addr = u16::from_le_bytes(cpu.tmp);
            let val = cpu.bus.mem_read(addr);
            cpu.gpr[Register::L] = val;
            T_PER_M
        });

        // M5: Load H from addr + 1
        self.pipeline.push_back(|cpu| {
            let addr = u16::from_le_bytes(cpu.tmp);
            let val = cpu.bus.mem_read(addr.wrapping_add(1));
            cpu.gpr[Register::H] = val;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn shld(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(1));

        // M4: Store L at addr
        self.pipeline.push_back(|cpu| {
            let addr = u16::from_le_bytes(cpu.tmp);
            let val = cpu.gpr[Register::L];
            cpu.bus.mem_write(addr, val);
            T_PER_M
        });

        // M5: Store H at addr + 1
        self.pipeline.push_back(|cpu| {
            let addr = u16::from_le_bytes(cpu.tmp);
            let val = cpu.gpr[Register::H];
            cpu.bus.mem_write(addr.wrapping_add(1), val);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn ldax_b(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline
            .push_back(|cpu| cpu.ldax_r_m2(Register::B, Register::C));

        NO_TCYCLES
    }

    pub(crate) fn ldax_d(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline
            .push_back(|cpu| cpu.ldax_r_m2(Register::D, Register::E));
        NO_TCYCLES
    }

    pub(crate) fn stax_b(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline
            .push_back(|cpu| cpu.stax_r_m2(Register::B, Register::C));

        NO_TCYCLES
    }

    pub(crate) fn stax_d(&mut self) -> TCycles {
        // M2: Store value in memory
        self.pipeline
            .push_back(|cpu| cpu.stax_r_m2(Register::D, Register::E));

        NO_TCYCLES
    }

    pub(crate) fn xchg(&mut self) -> TCycles {
        self.gpr.swap(Register::H as usize, Register::D as usize);
        self.gpr.swap(Register::L as usize, Register::E as usize);
        NO_TCYCLES
    }

    pub(crate) fn add_r(&mut self, src: Register) -> TCycles {
        self.update_flags_add(self.gpr[Register::A], self.gpr[src], false);
        self.gpr[Register::A] = self.gpr[Register::A].wrapping_add(self.gpr[src]);
        NO_TCYCLES
    }

    pub(crate) fn add_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            cpu.update_flags_add(cpu.gpr[Register::A], val, false);
            cpu.gpr[Register::A] = cpu.gpr[Register::A].wrapping_add(val);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn adi(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            cpu.update_flags_add(cpu.gpr[Register::A], cpu.tmp[0], false);
            cpu.gpr[Register::A] = cpu.gpr[Register::A].wrapping_add(cpu.tmp[0]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn adc_r(&mut self, src: Register) -> TCycles {
        let carry = self.flags.contains(Flags::CY) as u8;
        self.update_flags_add(self.gpr[Register::A], self.gpr[src], true);
        self.gpr[Register::A] = self.gpr[Register::A]
            .wrapping_add(self.gpr[src])
            .wrapping_add(carry);
        NO_TCYCLES
    }

    pub(crate) fn adc_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            let carry = cpu.flags.contains(Flags::CY) as u8;
            cpu.update_flags_add(cpu.gpr[Register::A], val, true);
            cpu.gpr[Register::A] = cpu.gpr[Register::A].wrapping_add(val).wrapping_add(carry);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn aci(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            let carry = cpu.flags.contains(Flags::CY) as u8;
            cpu.update_flags_add(cpu.gpr[Register::A], cpu.tmp[0], true);
            cpu.gpr[Register::A] = cpu.gpr[Register::A]
                .wrapping_add(cpu.tmp[0])
                .wrapping_add(carry);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn sub_r(&mut self, src: Register) -> TCycles {
        self.update_flags_sub(self.gpr[Register::A], self.gpr[src], false);
        self.gpr[Register::A] = self.gpr[Register::A].wrapping_sub(self.gpr[src]);
        NO_TCYCLES
    }

    pub(crate) fn sub_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            cpu.update_flags_sub(cpu.gpr[Register::A], val, false);
            cpu.gpr[Register::A] = cpu.gpr[Register::A].wrapping_sub(val);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn sui(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            cpu.update_flags_sub(cpu.gpr[Register::A], cpu.tmp[0], false);
            cpu.gpr[Register::A] = cpu.gpr[Register::A].wrapping_sub(cpu.tmp[0]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn sbb_r(&mut self, src: Register) -> TCycles {
        let carry = self.flags.contains(Flags::CY) as u8;
        self.update_flags_sub(self.gpr[Register::A], self.gpr[src], true);
        self.gpr[Register::A] = self.gpr[Register::A]
            .wrapping_sub(self.gpr[src])
            .wrapping_sub(carry);
        NO_TCYCLES
    }

    pub(crate) fn sbb_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            let carry = cpu.flags.contains(Flags::CY) as u8;

            cpu.update_flags_sub(cpu.gpr[Register::A], val, true);
            cpu.gpr[Register::A] = cpu.gpr[Register::A].wrapping_sub(val).wrapping_sub(carry);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn sbi(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            let carry = cpu.flags.contains(Flags::CY) as u8;
            cpu.update_flags_sub(cpu.gpr[Register::A], cpu.tmp[0], true);
            cpu.gpr[Register::A] = cpu.gpr[Register::A]
                .wrapping_sub(cpu.tmp[0])
                .wrapping_sub(carry);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn inr_r(&mut self, dest: Register) -> TCycles {
        self.update_flags_inc(self.gpr[dest]);
        self.gpr[dest] = self.gpr[dest].wrapping_add(1);
        ALU_TCYCLES
    }

    pub(crate) fn inr_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            cpu.update_flags_inc(val);
            cpu.tmp[0] = val.wrapping_add(1);
            T_PER_M
        });

        // M3: Write value back to memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            cpu.bus.mem_write(addr, cpu.tmp[0]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn dcr_r(&mut self, dest: Register) -> TCycles {
        self.update_flags_dec(self.gpr[dest]);
        self.gpr[dest] = self.gpr[dest].wrapping_sub(1);
        ALU_TCYCLES
    }

    pub(crate) fn dcr_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            cpu.update_flags_dec(val);
            cpu.tmp[0] = val.wrapping_sub(1);
            T_PER_M
        });

        // M3: Write value back to memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            cpu.bus.mem_write(addr, cpu.tmp[0]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn inx_r(&mut self, dest1: Register, dest2: Register) -> TCycles {
        let val = self.get_reg_pair(dest1, dest2).wrapping_add(1);
        self.set_reg_pair(dest1, dest2, val);
        ALU_TCYCLES
    }

    pub(crate) fn inx_sp(&mut self) -> TCycles {
        self.sp = self.sp.wrapping_add(1);
        ALU_TCYCLES
    }

    pub(crate) fn dcx_r(&mut self, dest1: Register, dest2: Register) -> TCycles {
        let val = self.get_reg_pair(dest1, dest2).wrapping_sub(1);
        self.set_reg_pair(dest1, dest2, val);
        ALU_TCYCLES
    }

    pub(crate) fn dcx_sp(&mut self) -> TCycles {
        self.sp = self.sp.wrapping_sub(1);
        ALU_TCYCLES
    }

    pub(crate) fn dad_b(&mut self) -> TCycles {
        // M2: Add low byte
        self.pipeline
            .push_back(|cpu| cpu.dad_r_m2(Register::B, Register::C));

        // M3: Add high byte (see dad_r_m2 comment)
        self.pipeline.push_back(|_| T_PER_M);

        NO_TCYCLES
    }

    pub(crate) fn dad_d(&mut self) -> TCycles {
        // M2: Add low byte
        self.pipeline
            .push_back(|cpu| cpu.dad_r_m2(Register::D, Register::E));

        // M3: Add high byte (see dad_r_m2 comment)
        self.pipeline.push_back(|_| T_PER_M);

        NO_TCYCLES
    }

    pub(crate) fn dad_h(&mut self) -> TCycles {
        // M2: Add low byte
        self.pipeline
            .push_back(|cpu| cpu.dad_r_m2(Register::H, Register::L));

        // M3: Add high byte (see dad_r_m2 comment)
        self.pipeline.push_back(|_| T_PER_M);

        NO_TCYCLES
    }

    pub(crate) fn dad_sp(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let pair1 = cpu.get_reg_pair(Register::H, Register::L) as u32;
            let res = pair1 + cpu.sp as u32;
            cpu.flags.set(Flags::CY, res > 0xFFFF);
            cpu.set_reg_pair(Register::H, Register::L, res as u16);
            T_PER_M
        });

        self.pipeline.push_back(|_| T_PER_M);
        NO_TCYCLES
    }

    pub(crate) fn daa(&mut self) -> TCycles {
        let mut val = 0;
        let nib_high = self.gpr[Register::A] >> 4;
        let nib_low = self.gpr[Register::A] & 0xF;

        if self.flags.contains(Flags::AC) || (nib_low > 9) {
            val += 0x06;
        }

        if self.flags.contains(Flags::CY) || (nib_high > 9) || (nib_high >= 9 && nib_low > 9) {
            val += 0x60;
            self.flags.insert(Flags::CY);
        }

        let res = self.gpr[Register::A].wrapping_add(val);
        self.update_flag_p(res);
        self.update_flag_z(res);
        self.update_flag_s(res);
        self.update_flag_ac_add(self.gpr[Register::A], val, false);

        self.gpr[Register::A] = res;

        NO_TCYCLES
    }

    pub(crate) fn ana_r(&mut self, src: Register) -> TCycles {
        self.update_flags_and(self.gpr[Register::A], self.gpr[src]);
        self.gpr[Register::A] &= self.gpr[src];
        NO_TCYCLES
    }

    pub(crate) fn ana_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            cpu.update_flags_and(cpu.gpr[Register::A], val);
            cpu.gpr[Register::A] &= val;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn ani(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            cpu.update_flags_and(cpu.gpr[Register::A], cpu.tmp[0]);
            cpu.gpr[Register::A] &= cpu.tmp[0];
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn xra_r(&mut self, src: Register) -> TCycles {
        let res = self.gpr[Register::A] ^ self.gpr[src];
        self.update_flags_or(res);
        self.gpr[Register::A] = res;
        NO_TCYCLES
    }

    pub(crate) fn xra_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            let res = cpu.gpr[Register::A] ^ val;
            cpu.update_flags_or(res);
            cpu.gpr[Register::A] = res;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn xri(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            let res = cpu.gpr[Register::A] ^ cpu.tmp[0];
            cpu.update_flags_or(res);
            cpu.gpr[Register::A] = res;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn ora_r(&mut self, src: Register) -> TCycles {
        let res = self.gpr[Register::A] | self.gpr[src];
        self.update_flags_or(res);
        self.gpr[Register::A] = res;
        NO_TCYCLES
    }

    pub(crate) fn ora_m(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            let res = cpu.gpr[Register::A] | val;
            cpu.update_flags_or(res);
            cpu.gpr[Register::A] = res;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn ori(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            let res = cpu.gpr[Register::A] | cpu.tmp[0];
            cpu.update_flags_or(res);
            cpu.gpr[Register::A] = res;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn cmp_r(&mut self, src: Register) -> TCycles {
        self.update_flags_cmp(self.gpr[Register::A], self.gpr[src]);
        NO_TCYCLES
    }

    pub(crate) fn cmp_m(&mut self) -> TCycles {
        // M2: Load value from memory
        self.pipeline.push_back(|cpu| {
            let addr = cpu.get_reg_pair(Register::H, Register::L);
            let val = cpu.bus.mem_read(addr);
            cpu.update_flags_cmp(cpu.gpr[Register::A], val);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn cpi(&mut self) -> TCycles {
        // M2: Fetch immediate value
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(0);
            cpu.update_flags_cmp(cpu.gpr[Register::A], cpu.tmp[0]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn rlc(&mut self) -> TCycles {
        let high_bit = (self.gpr[Register::A] >> 7) == 1;
        self.gpr[Register::A] <<= 1;

        if high_bit {
            self.gpr[Register::A] |= 1;
        } else {
            self.gpr[Register::A] &= !1;
        }

        self.flags.set(Flags::CY, high_bit);
        NO_TCYCLES
    }

    pub(crate) fn rrc(&mut self) -> TCycles {
        let low_bit = (self.gpr[Register::A] & 1) == 1;
        self.gpr[Register::A] >>= 1;

        if low_bit {
            self.gpr[Register::A] |= 1 << 7;
        } else {
            self.gpr[Register::A] &= !(1 << 7);
        }

        self.flags.set(Flags::CY, low_bit);
        NO_TCYCLES
    }

    pub(crate) fn ral(&mut self) -> TCycles {
        let high_bit = (self.gpr[Register::A] >> 7) == 1;
        self.gpr[Register::A] <<= 1;

        if self.flags.contains(Flags::CY) {
            self.gpr[Register::A] |= 1;
        } else {
            self.gpr[Register::A] &= !1;
        }

        self.flags.set(Flags::CY, high_bit);
        NO_TCYCLES
    }

    pub(crate) fn rar(&mut self) -> TCycles {
        let low_bit = (self.gpr[Register::A] & 1) == 1;
        self.gpr[Register::A] >>= 1;

        if self.flags.contains(Flags::CY) {
            self.gpr[Register::A] |= 1 << 7;
        } else {
            self.gpr[Register::A] &= !(1 << 7);
        }

        self.flags.set(Flags::CY, low_bit);
        NO_TCYCLES
    }

    pub(crate) fn cma(&mut self) -> TCycles {
        self.gpr[Register::A] = !self.gpr[Register::A];
        NO_TCYCLES
    }

    pub(crate) fn cmc(&mut self) -> TCycles {
        self.flags.toggle(Flags::CY);
        NO_TCYCLES
    }

    pub(crate) fn stc(&mut self) -> TCycles {
        self.flags.insert(Flags::CY);
        NO_TCYCLES
    }

    pub(crate) fn jmp(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of target addr then jump
        self.pipeline.push_back(|cpu| {
            cpu.fetch_operand(1);
            let addr = u16::from_le_bytes(cpu.tmp);
            cpu.pc = addr;
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn jnz(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate Z flag clear
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::Z, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::Z, false));

        NO_TCYCLES
    }

    pub(crate) fn jz(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate Z flag set
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::Z, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::Z, true));

        NO_TCYCLES
    }

    pub(crate) fn jnc(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate CY flag clear
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::CY, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::CY, false));

        NO_TCYCLES
    }

    pub(crate) fn jc(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate CY flag set
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::CY, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::CY, true));

        NO_TCYCLES
    }

    pub(crate) fn jpo(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate P flag clear
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::P, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::P, false));

        NO_TCYCLES
    }

    pub(crate) fn jpe(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate P flag set
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::P, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::P, true));

        NO_TCYCLES
    }

    pub(crate) fn jp(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate S flag clear
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::S, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::S, false));

        NO_TCYCLES
    }

    pub(crate) fn jm(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr, evaluate S flag set
        // M3 (non-8080 only): Fetch high byte of target addr only if branch taken
        self.pipeline.push_back(|cpu| cpu.jcc_m2(Flags::S, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.jcc_m3(Flags::S, true));

        NO_TCYCLES
    }

    pub(crate) fn call(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Fetch high byte of target addr
        self.pipeline.push_back(|cpu| cpu.fetch_operand(1));

        // M4: Push high byte of return addr
        self.pipeline.push_back(|cpu| cpu.call_m4());

        // M5: Push low byte of return addr, then jump
        self.pipeline.push_back(|cpu| cpu.call_m5());

        NO_TCYCLES
    }

    pub(crate) fn cnz(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::Z, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::Z, false));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cz(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::Z, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::Z, true));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cnc(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::CY, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::CY, false));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cc(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::CY, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::CY, true));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cpo(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::P, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::P, false));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cpe(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::P, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::P, true));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cp(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::S, false));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::S, false));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn cm(&mut self) -> TCycles {
        // M2: Fetch low byte of target addr
        // M3: (non-8080 only): Fetch high byte of target addr if branch taken
        self.pipeline.push_back(|cpu| cpu.ccc_m2(Flags::S, true));

        // M3 (8080 only): Fetch high byte of target addr regardless if branch taken
        #[cfg(feature = "i8080")]
        self.pipeline.push_back(|cpu| cpu.ccc_m3(Flags::S, true));

        // Queued in the above calls only if branch taken:
        // M4: Push high byte of return addr
        // M5: Push low byte of return addr, then jump

        NO_TCYCLES
    }

    pub(crate) fn ret(&mut self) -> TCycles {
        // M2: Pop low byte of return address from stack
        self.pipeline.push_back(|cpu| cpu.ret_m2());

        // M3: Pop high byte of return address from stack, then jump
        self.pipeline.push_back(|cpu| cpu.ret_m3());

        NO_TCYCLES
    }

    pub(crate) fn rnz(&mut self) -> TCycles {
        if !self.flags.contains(Flags::Z) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rz(&mut self) -> TCycles {
        if self.flags.contains(Flags::Z) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rnc(&mut self) -> TCycles {
        if !self.flags.contains(Flags::CY) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rc(&mut self) -> TCycles {
        if self.flags.contains(Flags::CY) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rpo(&mut self) -> TCycles {
        if !self.flags.contains(Flags::P) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rpe(&mut self) -> TCycles {
        if self.flags.contains(Flags::P) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rp(&mut self) -> TCycles {
        if !self.flags.contains(Flags::S) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rm(&mut self) -> TCycles {
        if self.flags.contains(Flags::S) {
            // M2: Pop low byte of return address from stack
            self.pipeline.push_back(|cpu| cpu.ret_m2());

            // M3: Pop high byte of return address from stack, then jump
            self.pipeline.push_back(|cpu| cpu.ret_m3());
        }

        ALU_TCYCLES
    }

    pub(crate) fn rst_0(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(0));

        NO_TCYCLES
    }

    pub(crate) fn rst_1(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(1));

        NO_TCYCLES
    }

    pub(crate) fn rst_2(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(2));

        NO_TCYCLES
    }

    pub(crate) fn rst_3(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(3));

        NO_TCYCLES
    }

    pub(crate) fn rst_4(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(4));

        NO_TCYCLES
    }

    pub(crate) fn rst_5(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(5));

        NO_TCYCLES
    }

    pub(crate) fn rst_6(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(6));

        NO_TCYCLES
    }

    pub(crate) fn rst_7(&mut self) -> TCycles {
        // M2: Push high byte of return address to stack
        self.pipeline.push_back(|cpu| cpu.rst_n_m2());

        // M3: Push low byte of return address to stack, then jump
        self.pipeline.push_back(|cpu| cpu.rst_n_m3(7));

        NO_TCYCLES
    }

    pub(crate) fn pchl(&mut self) -> TCycles {
        let addr = self.get_reg_pair(Register::H, Register::L);
        self.pc = addr;

        ALU_TCYCLES
    }

    pub(crate) fn push_b(&mut self) -> TCycles {
        // M2: Push high byte of register pair to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::B));

        // M3: Push low byte of register pair to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::C));

        NO_TCYCLES
    }

    pub(crate) fn push_d(&mut self) -> TCycles {
        // M2: Push high byte of register pair to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::D));

        // M3: Push low byte of register pair to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::E));

        NO_TCYCLES
    }

    pub(crate) fn push_h(&mut self) -> TCycles {
        // M2: Push high byte of register pair to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::H));

        // M3: Push low byte of register pair to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::L));

        NO_TCYCLES
    }

    pub(crate) fn push_psw(&mut self) -> TCycles {
        // M2: Push accumulator to stack
        self.pipeline.push_back(|cpu| cpu.push_r(Register::A));

        // M3: Push status flags (formatted as 8 bits) to stack
        self.pipeline.push_back(|cpu| {
            let psw = cpu.flags_to_u8();
            cpu.sp = cpu.sp.wrapping_sub(1);
            cpu.bus.mem_write(cpu.sp, psw);

            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn pop_b(&mut self) -> TCycles {
        // M2: Pop stack into low byte of register pair
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::C));

        // M3: Pop stack into high byte of register pair
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::B));

        NO_TCYCLES
    }

    pub(crate) fn pop_d(&mut self) -> TCycles {
        // M2: Pop stack into low byte of register pair
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::E));

        // M3: Pop stack into high byte of register pair
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::D));

        NO_TCYCLES
    }

    pub(crate) fn pop_h(&mut self) -> TCycles {
        // M2: Pop stack into low byte of register pair
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::L));

        // M3: Pop stack into high byte of register pair
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::H));

        NO_TCYCLES
    }

    pub(crate) fn pop_psw(&mut self) -> TCycles {
        // M2: Pop stack into status flags
        self.pipeline.push_back(|cpu| {
            let psw = cpu.bus.mem_read(cpu.sp);
            cpu.sp = cpu.sp.wrapping_add(1);
            cpu.flags.set(Flags::CY, (psw & 1) != 0);
            cpu.flags.set(Flags::P, (psw & (1 << 2)) != 0);
            cpu.flags.set(Flags::AC, (psw & (1 << 4)) != 0);
            cpu.flags.set(Flags::Z, (psw & (1 << 6)) != 0);
            cpu.flags.set(Flags::S, (psw & (1 << 7)) != 0);
            T_PER_M
        });

        // M3: Pop stack into accumulator
        self.pipeline.push_back(|cpu| cpu.pop_r(Register::A));

        NO_TCYCLES
    }

    pub(crate) fn xthl(&mut self) -> TCycles {
        // M2: Read stack into tmp (for L)
        self.pipeline.push_back(|cpu| {
            cpu.tmp[0] = cpu.bus.mem_read(cpu.sp);
            T_PER_M
        });

        // M3: Read stack + 1 into tmp (for H)
        self.pipeline.push_back(|cpu| {
            cpu.tmp[1] = cpu.bus.mem_read(cpu.sp.wrapping_add(1));
            T_PER_M
        });

        // M4: Replace stack with L
        self.pipeline.push_back(|cpu| {
            cpu.bus.mem_write(cpu.sp, cpu.gpr[Register::L]);
            T_PER_M
        });

        // M5: Replace stack + 1 with H, set HL pair
        self.pipeline.push_back(|cpu| {
            cpu.bus
                .mem_write(cpu.sp.wrapping_add(1), cpu.gpr[Register::H]);
            let val = u16::from_le_bytes(cpu.tmp);
            cpu.set_reg_pair(Register::H, Register::L, val);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn sphl(&mut self) -> TCycles {
        self.sp = self.get_reg_pair(Register::H, Register::L);
        ALU_TCYCLES
    }

    pub(crate) fn inp(&mut self) -> TCycles {
        // M2: Fetch immediate port number
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Read from port
        self.pipeline.push_back(|cpu| {
            cpu.gpr[Register::A] = cpu.bus.port_read(cpu.tmp[0]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn outp(&mut self) -> TCycles {
        // M2: Fetch immediate port number
        self.pipeline.push_back(|cpu| cpu.fetch_operand(0));

        // M3: Write to port
        self.pipeline.push_back(|cpu| {
            cpu.bus.port_write(cpu.tmp[0], cpu.gpr[Register::A]);
            T_PER_M
        });

        NO_TCYCLES
    }

    pub(crate) fn ei(&mut self) -> TCycles {
        self.iff = IffState::EnablePending;
        NO_TCYCLES
    }

    pub(crate) fn di(&mut self) -> TCycles {
        self.iff = IffState::Disabled;
        NO_TCYCLES
    }

    pub(crate) fn hlt(&mut self) -> TCycles {
        self.halt = true;

        // HLT is odd in that it is a single M-Cycle but takes quite a few T-Cycles
        3
    }

    pub(crate) fn nop(&mut self) -> TCycles {
        NO_TCYCLES
    }

    // Intel 8080 doesn't have traps or any defined behavior for undefined opcodes.
    // Can't find any unofficial docs to see if there is still deterministic behavior.
    // Might be fun to investigate more in the future.
    pub(crate) fn undef(&mut self, opcode: crate::Opcode) -> TCycles {
        println!(
            "Encountered undefined opcode: {opcode:?} at PC={:04X}",
            self.pc
        );
        NO_TCYCLES
    }
}
