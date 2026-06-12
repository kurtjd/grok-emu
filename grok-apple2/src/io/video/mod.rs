mod ntsc;

use grok_6502::bus::Bus;
use ntsc::Ntsc;

const BLOCK_ROWS: usize = 24;
const BLOCK_COLS: usize = 40;
const BLOCK_WIDTH: usize = 7;
const BLOCK_HEIGHT: usize = 8;
const FLASH_FRAMES: u32 = 15;

pub(crate) const CHAR_ROM_SIZE: usize = 0x800;
pub(crate) const VSCAN_MAX: u16 = 262;
pub(crate) const HSCAN_MAX: u16 = 65;

mod soft_switch {
    pub const GFX_MODE: u16 = 0xC050;
    pub const TXT_MODE: u16 = 0xC051;
    pub const SINGLE_MODE: u16 = 0xC052;
    pub const MIXED_MODE: u16 = 0xC053;
    pub const PG1_MODE: u16 = 0xC054;
    pub const PG2_MODE: u16 = 0xC055;
    pub const LORES_MODE: u16 = 0xC056;
    pub const HIRES_MODE: u16 = 0xC057;
}

type FrameBuf = [[FrameByte; BLOCK_COLS]; BLOCK_ROWS * BLOCK_HEIGHT];

// I know enough about how the Apple II actually generates video to know this is not the best approach,
// and some NTSC specifics have leaked into this, but not enough to know something better just yet
//
// Have some ideas, and would be cool to come back and completely decouple frame generation
// from renderer, and have more accurate NTSC coloring, but probably won't be anytime soon
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FrameByte {
    Lores(u8),
    Hires(u8),
    Text(u8),
}

impl FrameByte {
    fn val(self) -> u8 {
        match self {
            FrameByte::Lores(v) | FrameByte::Hires(v) | FrameByte::Text(v) => v,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Frame {
    buf: FrameBuf,
    idx: usize,
}

impl Frame {
    fn new() -> Self {
        Frame {
            buf: [[FrameByte::Text(0); BLOCK_COLS]; BLOCK_ROWS * BLOCK_HEIGHT],
            idx: 0,
        }
    }

    fn latch_next(&mut self, byte: FrameByte) {
        self.buf[self.row()][self.col()] = byte;

        self.idx += 1;
        if self.idx >= BLOCK_COLS * BLOCK_ROWS * BLOCK_HEIGHT {
            self.idx = 0;
        }
    }

    fn block_row(&self) -> usize {
        (self.idx / BLOCK_COLS) % BLOCK_HEIGHT
    }

    fn col(&self) -> usize {
        self.idx % BLOCK_COLS
    }

    fn row(&self) -> usize {
        self.idx / BLOCK_COLS
    }
}

pub struct Video {
    phase1: bool,
    char_rom: [u8; CHAR_ROM_SIZE],
    renderer: Ntsc,
    frame: Frame,
    frame_count: u32,
    flash: bool,
    txt_mode: bool,
    hires_mode: bool,
    mixed_mode: bool,
    use_pg2: bool,
}

impl Video {
    pub(crate) fn new(char_rom: [u8; CHAR_ROM_SIZE]) -> Self {
        // We do this to be consistent with how hires pixels are represented.
        // Basically, hires bytes represent dots in reverse order,
        // So we reverse the character rom bytes to match this.
        //
        // The idea is we can then just draw text in the same way we draw hires
        let char_rom = char_rom.map(|byte| byte.reverse_bits() >> 1);

        Video {
            phase1: true,
            char_rom,
            renderer: Ntsc::new(),
            frame: Frame::new(),
            frame_count: 0,
            flash: false,
            txt_mode: true,
            hires_mode: false,
            mixed_mode: false,
            use_pg2: false,
        }
    }

    pub(crate) fn tick(&mut self, vscan: u16, hscan: u16, bus: &mut dyn Bus) {
        // Sather 3-12
        // The vertical scan counter is in range 011111010-111111111,
        // so we just add 0xFA to offset it into that range always
        let vscan = vscan + 0xFA;

        // Sather 5-13
        // These two bits tell us if we are in "HIRES time" in mixed mode,
        // which is useful for determining if we are drawing a character or hires byte
        let (v2, v4) = ((vscan >> 5) & 1, (vscan >> 7) & 1);
        let hires_time = (v4 == 0) || (v2 == 0);

        if self.phase1 {
            // Sather 3-12
            let addr = self.vram_base_addr(vscan, hscan);

            let addr = if self.mixed_mode && self.hires_mode {
                if hires_time {
                    self.vram_hires_addr(addr, vscan)
                } else {
                    self.vram_lores_addr(addr, hscan)
                }
            } else if self.hires_mode {
                self.vram_hires_addr(addr, vscan)
            } else {
                self.vram_lores_addr(addr, hscan)
            };

            // It's important we always read from the bus here even if not in display region
            // because software can take advantage of the floating bus to perform vapor locking:
            //
            // http://www.deater.net/weave/vmwprod/megademo/vapor_lock.html
            bus.start_read(addr);
        } else if !self.in_hbl(hscan) && !self.in_vbl(vscan) {
            let data = bus.data();

            if self.txt_mode || (!hires_time && self.mixed_mode) {
                self.latch_char_byte(data);
            } else if self.hires_mode {
                self.frame.latch_next(FrameByte::Hires(data));
            } else {
                self.latch_lores_byte(data);
            }
        }

        self.phase1 = !self.phase1;
    }

    pub(crate) fn decode(&mut self, bus: &dyn Bus) {
        match bus.addr() {
            soft_switch::TXT_MODE => {
                self.txt_mode = true;
                self.hires_mode = false;
            }
            soft_switch::GFX_MODE => self.txt_mode = false,
            soft_switch::SINGLE_MODE => self.mixed_mode = false,
            soft_switch::MIXED_MODE => self.mixed_mode = true,
            soft_switch::PG1_MODE => self.use_pg2 = false,
            soft_switch::PG2_MODE => self.use_pg2 = true,
            soft_switch::LORES_MODE => self.hires_mode = false,
            soft_switch::HIRES_MODE => self.hires_mode = true,
            _ => {}
        }
    }

    pub(crate) fn render(&mut self) -> &[u32] {
        self.handle_flash();
        self.renderer.render(&self.frame.buf)
    }

    fn in_hbl(&self, hscan: u16) -> bool {
        let (h3, h4, h5) = ((hscan >> 3) & 1, (hscan >> 4) & 1, (hscan >> 5) & 1);

        // Sather 8-9
        (h5 == 0) && ((h3 == 0) || (h4 == 0))
    }

    fn in_vbl(&self, vscan: u16) -> bool {
        let (v3, v4) = ((vscan >> 6) & 1, (vscan >> 7) & 1);

        // Sather 8-9
        v4 != 0 && v3 != 0
    }

    // This builds the upper 3 bits of the video RAM byte we are about to address in hires mode
    fn vram_hires_addr(&self, addr: u16, vscan: u16) -> u16 {
        let (va, vb, vc) = (vscan & 1, (vscan >> 1) & 1, (vscan >> 2) & 1);

        // Sather 5-7
        let page = if self.use_pg2 { 0b10 } else { 0b01 };
        (page << 13) | (vc << 12) | (vb << 11) | (va << 10) | addr
    }

    // This builds the upper 3 bits of the video RAM byte we are about to address in lores/text mode
    fn vram_lores_addr(&self, addr: u16, hscan: u16) -> u16 {
        // Sather 5-7
        let page = if self.use_pg2 { 0b10 } else { 0b01 };
        ((self.in_hbl(hscan) as u16) << 12) | (page << 10) | addr
    }

    // This builds the lower 10 bits of the video RAM byte we are about to address
    // which is independent of mode
    fn vram_base_addr(&self, vscan: u16, hscan: u16) -> u16 {
        // Sather 3-2
        let (h0, h1, h2, h3, h4, h5) = (
            hscan & 1,
            (hscan >> 1) & 1,
            (hscan >> 2) & 1,
            (hscan >> 3) & 1,
            (hscan >> 4) & 1,
            (hscan >> 5) & 1,
        );
        let (v0, v1, v2, v3, v4) = (
            (vscan >> 3) & 1,
            (vscan >> 4) & 1,
            (vscan >> 5) & 1,
            (vscan >> 6) & 1,
            (vscan >> 7) & 1,
        );

        // Sather 5-9
        let a = (h5 << 2) | (h4 << 1) | h3;
        let b = (v4 << 3) | (v3 << 2) | (v4 << 1) | v3;
        let sum = (0b1101 + a + b) & 0x0F;

        // Sather 5-7
        (v2 << 9) | (v1 << 8) | (v0 << 7) | (sum << 3) | (h2 << 2) | (h1 << 1) | h0
    }

    fn latch_char_byte(&mut self, byte: u8) {
        // Mask off the upper two bits as they don't affect address
        // Then multiply by 8 (since each character is represented by 8 bytes)
        let char_byte_addr = ((byte & 0x3F) as usize * BLOCK_HEIGHT) + self.frame.block_row();

        // 7th bit tells us if in normal mode
        // 6th bit tells us if in flash or inverse mode (if bit 7 is not set)
        let char_byte = if (byte & (1 << 7) == 0) && (byte & (1 << 6) == 0 || self.flash) {
            // We only invert the 7 LSBs because the MSB is unused and don't want it mistaken as a phase shift
            self.char_rom[char_byte_addr] ^ 0x7F
        } else {
            self.char_rom[char_byte_addr]
        };

        // Still a bit unsure about this one. Apparently Apple II will turn off color burst
        // for text in mixed mode, but also most TVs couldn't actually switch in time,
        // so we still see color artifacts on text (even in mixed lores mode).
        //
        // So, if we aren't explictly in text mode we treat text as hires so we still get color artifacts later.
        // This seems to be the most "typical" behavior
        let byte = if self.txt_mode {
            FrameByte::Text(char_byte)
        } else {
            FrameByte::Hires(char_byte)
        };
        self.frame.latch_next(byte);
    }

    fn latch_lores_byte(&mut self, byte: u8) {
        // We use the high nybble as the bit pattern when in the top of the block
        // And the low nybble when in the bottom of the block
        let byte = if self.frame.block_row() < (BLOCK_HEIGHT / 2) {
            byte & 0x0F
        } else {
            byte >> 4
        };
        self.frame.latch_next(FrameByte::Lores(byte));
    }

    fn handle_flash(&mut self) {
        self.frame_count += 1;
        if self.frame_count >= FLASH_FRAMES {
            self.flash = !self.flash;
            self.frame_count = 0;
        }
    }
}
