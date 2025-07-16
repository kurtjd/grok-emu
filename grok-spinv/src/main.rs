//! Complete Space Invaders (1978) arcade machine emulator
mod bus;
mod input_reg;
mod shift_reg;
mod sound_reg;

use bus::Bus;
use grok_80::{Cpu, Opcode};

const CPU_FREQ_HZ: u64 = 2_000_000;
const FRAME_RATE_HZ: u64 = 60;
const HALF_VBLANK_CYCLES: u64 = (CPU_FREQ_HZ / FRAME_RATE_HZ) / 2;

fn main() {
    let mut bus = Bus::new();
    let mut cpu: Cpu<Bus> = Cpu::new();

    let mut tcycles = 0;
    let mut vblank = false;

    loop {
        tcycles += cpu.step(&mut bus).expect("Unexpected HLT!").tcycles;

        if tcycles >= HALF_VBLANK_CYCLES {
            let opcode = if vblank { Opcode::RST_2 } else { Opcode::RST_1 };
            cpu.interrupt(opcode);

            vblank = !vblank;
            tcycles -= HALF_VBLANK_CYCLES;
        }
    }
}
