use crate::*;

pub(crate) struct Opcode {
    pub(crate) instr: Instruction,
    pub(crate) mode: AddrMode,
}

pub(crate) static OPCODES: [Opcode; 0x100] = [
    // $00-$0F
    Opcode {
        instr: Instruction::Misc(Cpu::brk),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::asl),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Push(Cpu::php),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::asl),
        mode: AddrMode::Acm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::anc),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::asl),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::Abs0,
    },
    // $10-$1F
    Opcode {
        instr: Instruction::Branch(Cpu::bpl),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::asl),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::clc),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ora),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::asl),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::slo),
        mode: AddrMode::AbsX,
    },
    // $20-$2F
    Opcode {
        instr: Instruction::Misc(Cpu::jsr),
        // Technically more like an ABS0 but follows dispatch path better as Imp0
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::bit),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::rol),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Pull(Cpu::plp),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::rol),
        mode: AddrMode::Acm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::anc),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::bit),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::rol),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::Abs0,
    },
    // $30-$3F
    Opcode {
        instr: Instruction::Branch(Cpu::bmi),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::rol),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::sec),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::and),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::rol),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rla),
        mode: AddrMode::AbsX,
    },
    // $40-$4F
    Opcode {
        instr: Instruction::Misc(Cpu::rti),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::lsr),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Push(Cpu::pha),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::lsr),
        mode: AddrMode::Acm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::alr),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Jmp(Cpu::jmp),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::lsr),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::Abs0,
    },
    // $50-$5F
    Opcode {
        instr: Instruction::Branch(Cpu::bvc),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::lsr),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::cli),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::eor),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::lsr),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sre),
        mode: AddrMode::AbsX,
    },
    // $60-$6F
    Opcode {
        instr: Instruction::Misc(Cpu::rts),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::ror),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Pull(Cpu::pla),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::ror),
        mode: AddrMode::Acm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::arr),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Jmp(Cpu::jmp),
        mode: AddrMode::Ind0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::ror),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::Abs0,
    },
    // $70-$7F
    Opcode {
        instr: Instruction::Branch(Cpu::bvs),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::ror),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::sei),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::adc),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::ror),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::rra),
        mode: AddrMode::AbsX,
    },
    // $80-$8F
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sax),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sty),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::stx),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sax),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::dey),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::txa),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::ane),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sty),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::stx),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sax),
        mode: AddrMode::Abs0,
    },
    // $90-$9F
    Opcode {
        instr: Instruction::Branch(Cpu::bcc),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sha),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sty),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Write(Cpu::stx),
        mode: AddrMode::ZpgY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sax),
        mode: AddrMode::ZpgY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::tya),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::txs),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::tas),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::shy),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Write(Cpu::sta),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::shx),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sha),
        mode: AddrMode::AbsY,
    },
    // $A0-$AF
    Opcode {
        instr: Instruction::Read(Cpu::ldy),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldx),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lax),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldy),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldx),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lax),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::tay),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::tax),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lxa),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldy),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldx),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lax),
        mode: AddrMode::Abs0,
    },
    // $B0-$BF
    Opcode {
        instr: Instruction::Branch(Cpu::bcs),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lax),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldy),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldx),
        mode: AddrMode::ZpgY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lax),
        mode: AddrMode::ZpgY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::clv),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::tsx),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::las),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldy),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::lda),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::ldx),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::lax),
        mode: AddrMode::AbsY,
    },
    // $C0-$CF
    Opcode {
        instr: Instruction::Read(Cpu::cpy),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cpy),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::dec),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::iny),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::dex),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::sbx),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cpy),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::dec),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::Abs0,
    },
    // $D0-$DF
    Opcode {
        instr: Instruction::Branch(Cpu::bne),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::dec),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::cld),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cmp),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::dec),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::dcp),
        mode: AddrMode::AbsX,
    },
    // $E0-$EF
    Opcode {
        instr: Instruction::Read(Cpu::cpx),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::IndX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cpx),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::inc),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::Zpg0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::inx),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::usb),
        mode: AddrMode::Imm0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::cpx),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::inc),
        mode: AddrMode::Abs0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::Abs0,
    },
    // $F0-$FF
    Opcode {
        instr: Instruction::Branch(Cpu::beq),
        mode: AddrMode::Rel0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::jam),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::IndY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::inc),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::ZpgX,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::sed),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::SingleByte(Cpu::nop),
        mode: AddrMode::Imp0,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::AbsY,
    },
    Opcode {
        instr: Instruction::Read(Cpu::nop_read),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Read(Cpu::sbc),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Rmw(Cpu::inc),
        mode: AddrMode::AbsX,
    },
    Opcode {
        instr: Instruction::Misc(Cpu::isc),
        mode: AddrMode::AbsX,
    },
];

impl Cpu {
    // Commonly performed by quite a few instructions
    fn update_zn_flags(&mut self, result: u8) {
        self.registers.p.set(StatusFlags::Z, result == 0);
        self.registers.p.set(StatusFlags::N, result & 0x80 != 0);
    }

    fn compare(&mut self, reg: u8, data: u8) {
        let result = reg.wrapping_sub(data);
        self.update_zn_flags(result);
        self.registers.p.set(StatusFlags::C, reg >= data);
    }

    // SingleByte
    fn clc(&mut self) {
        self.registers.p &= !StatusFlags::C;
    }
    fn cld(&mut self) {
        self.registers.p &= !StatusFlags::D;
    }
    fn cli(&mut self) {
        self.registers.p &= !StatusFlags::I;
    }
    fn clv(&mut self) {
        self.registers.p &= !StatusFlags::V;
    }
    fn dex(&mut self) {
        self.registers.x = self.registers.x.wrapping_sub(1);
        self.update_zn_flags(self.registers.x);
    }
    fn dey(&mut self) {
        self.registers.y = self.registers.y.wrapping_sub(1);
        self.update_zn_flags(self.registers.y);
    }
    fn inx(&mut self) {
        self.registers.x = self.registers.x.wrapping_add(1);
        self.update_zn_flags(self.registers.x);
    }
    fn iny(&mut self) {
        self.registers.y = self.registers.y.wrapping_add(1);
        self.update_zn_flags(self.registers.y);
    }
    fn sec(&mut self) {
        self.registers.p |= StatusFlags::C;
    }
    fn sed(&mut self) {
        self.registers.p |= StatusFlags::D;
    }
    fn sei(&mut self) {
        self.registers.p |= StatusFlags::I;
    }
    fn tax(&mut self) {
        self.update_zn_flags(self.registers.a);
        self.registers.x = self.registers.a;
    }
    fn tay(&mut self) {
        self.update_zn_flags(self.registers.a);
        self.registers.y = self.registers.a;
    }
    fn tsx(&mut self) {
        self.update_zn_flags(self.registers.s);
        self.registers.x = self.registers.s;
    }
    fn txa(&mut self) {
        self.update_zn_flags(self.registers.x);
        self.registers.a = self.registers.x;
    }
    fn txs(&mut self) {
        self.registers.s = self.registers.x;
    }
    fn tya(&mut self) {
        self.update_zn_flags(self.registers.y);
        self.registers.a = self.registers.y;
    }

    // Read
    fn adc(&mut self, data: u8) {
        let carry = self.registers.p.contains(StatusFlags::C) as u16;
        let a = self.registers.a as u16;
        let d = data as u16;

        let bsum = a + d + carry;
        let mut sum = if self.registers.p.contains(StatusFlags::D) {
            // Add low nibbles
            let mut lo = carry + (a & 0x0F) + (d & 0x0F);

            // Perform correction and set the carry bit
            if lo > 0x09 {
                lo = ((lo + 0x06) & 0x0F) | 0x10;
            }

            // Add high nibbles plus corrected low nibble
            (a & 0xF0) + (d & 0xF0) + lo
        } else {
            bsum
        };

        // Must set negative and overflow flags before correcting high nibble
        self.registers.p.set(StatusFlags::N, sum & 0x80 != 0);
        self.registers
            .p
            .set(StatusFlags::V, (!(a ^ d) & (a ^ sum) & 0x80) != 0);

        // Correct high nibble
        if self.registers.p.contains(StatusFlags::D) && sum > 0x9F {
            sum += 0x60;
        }

        // Now set carry flag
        self.registers.p.set(StatusFlags::C, sum > 0xFF);
        // Zero flag is always set based on binary addition
        self.registers.p.set(StatusFlags::Z, bsum as u8 == 0);

        self.registers.a = sum as u8;
    }
    fn and(&mut self, data: u8) {
        self.registers.a &= data;
        self.update_zn_flags(self.registers.a);
    }
    fn bit(&mut self, data: u8) {
        self.registers
            .p
            .set(StatusFlags::Z, self.registers.a & data == 0);
        self.registers
            .p
            .set(StatusFlags::V, data & StatusFlags::V.bits() != 0);
        self.registers
            .p
            .set(StatusFlags::N, data & StatusFlags::N.bits() != 0);
    }
    fn cmp(&mut self, data: u8) {
        self.compare(self.registers.a, data);
    }
    fn cpx(&mut self, data: u8) {
        self.compare(self.registers.x, data);
    }
    fn cpy(&mut self, data: u8) {
        self.compare(self.registers.y, data);
    }
    fn eor(&mut self, data: u8) {
        self.registers.a ^= data;
        self.update_zn_flags(self.registers.a);
    }
    fn lda(&mut self, data: u8) {
        self.registers.a = data;
        self.update_zn_flags(self.registers.a);
    }
    fn ldx(&mut self, data: u8) {
        self.registers.x = data;
        self.update_zn_flags(self.registers.x);
    }
    fn ldy(&mut self, data: u8) {
        self.registers.y = data;
        self.update_zn_flags(self.registers.y);
    }
    fn ora(&mut self, data: u8) {
        self.registers.a |= data;
        self.update_zn_flags(self.registers.a);
    }
    fn sbc(&mut self, data: u8) {
        // We subtract the inverted carry bit
        let borrow = !self.registers.p.contains(StatusFlags::C) as u16;
        let a = self.registers.a as u16;
        let d = data as u16;

        let bsub = a.wrapping_sub(d).wrapping_sub(borrow);
        let sum = if self.registers.p.contains(StatusFlags::D) {
            // Subtract low nibbles and inverted carry
            let mut lo = (a & 0x0F).wrapping_sub(d & 0x0F).wrapping_sub(borrow);

            // Perform correction
            // 'fix' represents if the low nibble underflowed
            let mut fix = 0;
            if lo & 0x10 != 0 {
                lo = lo.wrapping_sub(0x06) & 0x0F;
                fix = 1;
            }

            // Subtract high nibbles and 1 if corrected lower nibble underflowed
            let mut hi = (a >> 4).wrapping_sub(d >> 4).wrapping_sub(fix);
            if hi & 0x10 != 0 {
                hi = hi.wrapping_sub(0x06);
            }

            // Merge high and low nibbles
            (hi << 4) | (lo & 0x0F)
        } else {
            bsub
        };

        // Update flags (SBC always updates flags based on binary result)
        // Thus decimal mode has no effect here
        self.update_zn_flags(bsub as u8);
        // We check overflow based on the 1's complement of the operand
        self.registers
            .p
            .set(StatusFlags::V, (a ^ d) & (a ^ bsub) & 0x80 != 0);
        // In SBC case, carry is set if a borrow did NOT occur
        self.registers.p.set(StatusFlags::C, bsub <= 0xFF);

        self.registers.a = sum as u8;
    }

    // Write
    fn sta(&mut self) -> u8 {
        self.registers.a
    }
    fn stx(&mut self) -> u8 {
        self.registers.x
    }
    fn sty(&mut self) -> u8 {
        self.registers.y
    }

    // Rmw
    fn asl(&mut self, data: u8) -> u8 {
        self.registers.p.set(StatusFlags::C, data & 0x80 != 0);
        let result = data << 1;
        self.update_zn_flags(result);
        result
    }
    fn dec(&mut self, data: u8) -> u8 {
        let result = data.wrapping_sub(1);
        self.update_zn_flags(result);
        result
    }
    fn inc(&mut self, data: u8) -> u8 {
        let result = data.wrapping_add(1);
        self.update_zn_flags(result);
        result
    }
    fn lsr(&mut self, data: u8) -> u8 {
        self.registers.p.set(StatusFlags::C, data & 0x01 != 0);
        let result = data >> 1;
        self.update_zn_flags(result);
        result
    }
    fn rol(&mut self, data: u8) -> u8 {
        let old_carry = self.registers.p.contains(StatusFlags::C) as u8;
        self.registers.p.set(StatusFlags::C, data & 0x80 != 0);
        let result = (data << 1) | old_carry;
        self.update_zn_flags(result);
        result
    }
    fn ror(&mut self, data: u8) -> u8 {
        let old_carry = self.registers.p.contains(StatusFlags::C) as u8;
        self.registers.p.set(StatusFlags::C, data & 0x01 != 0);
        let result = (data >> 1) | (old_carry << 7);
        self.update_zn_flags(result);
        result
    }

    // Branch
    fn bcc(&mut self) -> bool {
        !self.registers.p.contains(StatusFlags::C)
    }
    fn bcs(&mut self) -> bool {
        self.registers.p.contains(StatusFlags::C)
    }
    fn beq(&mut self) -> bool {
        self.registers.p.contains(StatusFlags::Z)
    }
    fn bmi(&mut self) -> bool {
        self.registers.p.contains(StatusFlags::N)
    }
    fn bne(&mut self) -> bool {
        !self.registers.p.contains(StatusFlags::Z)
    }
    fn bpl(&mut self) -> bool {
        !self.registers.p.contains(StatusFlags::N)
    }
    fn bvc(&mut self) -> bool {
        !self.registers.p.contains(StatusFlags::V)
    }
    fn bvs(&mut self) -> bool {
        self.registers.p.contains(StatusFlags::V)
    }

    // Push
    fn pha(&mut self) -> u8 {
        self.registers.a
    }
    fn php(&mut self) -> u8 {
        (self.registers.p | StatusFlags::B).bits()
    }

    // Pull
    fn pla(&mut self, data: u8) {
        self.registers.a = data;
        self.update_zn_flags(self.registers.a);
    }
    fn plp(&mut self, data: u8) {
        // We should ignore the Break and Extension flags from the pop
        self.registers.p &= StatusFlags::B | StatusFlags::E;
        self.registers.p |=
            StatusFlags::from_bits_truncate(data) & !(StatusFlags::B | StatusFlags::E);
    }

    // Jump
    fn jmp(&mut self, addr: u16) {
        self.registers.pc = addr;
    }

    // Misc
    // These instructions don't fit cleanly into the dispatch paths,
    // so they manually cycle through their state machine and handle bus transactions
    fn brk(&mut self, bus: &mut dyn Bus) {
        match self.hcycle {
            // T1 (Dummy read)
            3 => self.fetch_pc(bus),
            4 => (),

            // T2 (Push PC high byte to stack)
            5 => self.stack_push(bus, (self.registers.pc >> 8) as u8),
            6 => (),

            // T3 (Push PC low byte to stack)
            7 => self.stack_push(bus, (self.registers.pc & 0xFF) as u8),
            8 => (),

            // T4 (Push status reg to stack)
            9 => {
                let data = self.php();
                self.stack_push(bus, data);
            }
            10 => (),

            // Note: This currently hardcodes interrupt vector,
            // but will need to support NMI vector as well
            // (reset vector is not needed here since it is covered manually by Reset state)

            // T5 (Fetch low byte from interrupt vector)
            11 => bus.start_read(INTR_VECTOR),
            12 => self.registers.internal.scratch[0] = bus.data(),

            // T6 (Fetch high byte from interrupt vector + jump)
            13 => bus.start_read(INTR_VECTOR + 1),
            14 => {
                self.registers.internal.scratch[1] = bus.data();
                self.registers.pc = u16::from_le_bytes(self.registers.internal.scratch);
                self.registers.p |= StatusFlags::I;
                self.end_instruction(bus);
            }

            _ => unreachable!(),
        }
    }
    fn jsr(&mut self, bus: &mut dyn Bus) {
        match self.hcycle {
            // T1 (Fetch low byte of subroutine address)
            3 => self.fetch_pc(bus),
            4 => self.registers.internal.scratch[0] = bus.data(),

            // T2 (Dummy read)
            5 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
            6 => (),

            // T3 (Push PC high byte to stack)
            7 => self.stack_push(bus, (self.registers.pc >> 8) as u8),
            8 => (),

            // T4 (Push PC low byte to stack)
            9 => self.stack_push(bus, (self.registers.pc & 0xFF) as u8),
            10 => (),

            // T5 (Fetch high byte of subroutine address)
            11 => bus.start_read(self.registers.pc),
            12 => {
                self.registers.internal.scratch[1] = bus.data();
                self.registers.pc = u16::from_le_bytes(self.registers.internal.scratch);
                self.end_instruction(bus);
            }

            _ => unreachable!(),
        }
    }
    fn rti(&mut self, bus: &mut dyn Bus) {
        match self.hcycle {
            // T1 (Dummy read)
            3 => self.fetch_pc(bus),
            4 => (),

            // T2 (Dummy read)
            5 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
            6 => (),

            // T3 (Pop status reg from stack)
            7 => self.stack_pop(bus),
            8 => {
                let data = bus.data();
                self.plp(data);
            }

            // T4 (Pop PC low byte from stack)
            9 => self.stack_pop(bus),
            10 => self.registers.internal.scratch[0] = bus.data(),

            // T5 (Pop PC high byte from stack)
            11 => self.stack_pop(bus),
            12 => {
                self.registers.internal.scratch[1] = bus.data();
                self.registers.pc = u16::from_le_bytes(self.registers.internal.scratch);
                self.end_instruction(bus);
            }

            _ => unreachable!(),
        }
    }
    fn rts(&mut self, bus: &mut dyn Bus) {
        match self.hcycle {
            // T1 (Dummy read)
            3 => self.fetch_pc(bus),
            4 => (),

            // T2 (Dummy read)
            5 => bus.start_read(STACK_OFFSET + self.registers.s as u16),
            6 => (),

            // T3 (Pop PC low byte from stack)
            7 => self.stack_pop(bus),
            8 => self.registers.internal.scratch[0] = bus.data(),

            // T4 (Pop PC high byte from stack)
            9 => self.stack_pop(bus),
            10 => self.registers.internal.scratch[1] = bus.data(),

            // T5 (Dummy read from popped PC + jump)
            11 => {
                self.registers.pc = u16::from_le_bytes(self.registers.internal.scratch);
                bus.start_read(self.registers.pc);
            }
            12 => {
                // Increment happens after dummy read
                self.registers.pc = self.registers.pc.wrapping_add(1);
                self.end_instruction(bus);
            }

            _ => unreachable!(),
        }
    }

    // No-op
    fn nop(&mut self) {
        // Intentionally do nothing
    }
    fn nop_read(&mut self, _data: u8) {
        // Intentionally do nothing
    }

    // Illegal
    fn alr(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn anc(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn ane(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn arr(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn dcp(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn isc(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn jam(&mut self, _bus: &mut dyn Bus) {
        self.halt();
    }
    fn las(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn lax(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn lxa(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn rla(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn rra(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn sax(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn sbx(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn sha(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn shx(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn shy(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn slo(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn sre(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn tas(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
    fn usb(&mut self, _bus: &mut dyn Bus) {
        todo!()
    }
}
