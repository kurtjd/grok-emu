use crate::{BusHandlerZ80, Cpu};

#[derive(Copy, Clone, Debug, num_enum::FromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum OpcodeCB {
    #[default]
    RES_0_B,
}

impl<B: BusHandlerZ80> Cpu<B> {
    pub(crate) fn execute_prefix_cb(&mut self, opcode: OpcodeCB, bus: &mut B) {
        match opcode {
            OpcodeCB::RES_0_B => {
                self.reg.b &= !(1 << 0);
                self.end_instruction(bus);
            }
        }
    }
}
