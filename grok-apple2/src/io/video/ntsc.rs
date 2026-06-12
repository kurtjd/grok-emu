use super::{BLOCK_COLS, BLOCK_HEIGHT, BLOCK_ROWS, BLOCK_WIDTH, FrameBuf, FrameByte};

mod color {
    // All
    pub const BLACK: u32 = 0x000000;
    pub const WHITE: u32 = 0xFFFFFF;

    // LORES
    pub const MAGENTA: u32 = 0x901740;
    pub const DARK_BLUE: u32 = 0x402CA5;
    pub const PURPLE: u32 = 0xD043E5;
    pub const DARK_GREEN: u32 = 0x006940;
    pub const GREY1: u32 = 0x808080;
    pub const BLUE: u32 = 0x2F95E5;
    pub const LIGHT_BLUE: u32 = 0xBFABFF;
    pub const BROWN: u32 = 0x405400;
    pub const ORANGE: u32 = 0xD06A1A;
    pub const GREY2: u32 = 0x808080;
    pub const PINK: u32 = 0xFF96BF;
    pub const LIGHT_GREEN: u32 = 0x2FBC1A;
    pub const YELLOW: u32 = 0xBFD35A;
    pub const AQUA: u32 = 0x6FE8BF;

    pub const LORES_MAP: [u32; 16] = [
        BLACK,
        MAGENTA,
        DARK_BLUE,
        PURPLE,
        DARK_GREEN,
        GREY1,
        BLUE,
        LIGHT_BLUE,
        BROWN,
        ORANGE,
        GREY2,
        PINK,
        LIGHT_GREEN,
        YELLOW,
        AQUA,
        WHITE,
    ];

    // HIRES
    pub const HIRES_BLUE: u32 = 0x4BB8F1;
    pub const HIRES_ORANGE: u32 = 0xE6792E;
    pub const HIRES_VIOLET: u32 = 0xD660EF;
    pub const HIRES_GREEN: u32 = 0x68E043;
}

pub(super) struct Ntsc {
    buf: [u32; BLOCK_COLS * BLOCK_WIDTH * BLOCK_ROWS * BLOCK_HEIGHT],
}

impl Ntsc {
    pub(super) fn new() -> Self {
        Ntsc {
            buf: [0; BLOCK_COLS * BLOCK_WIDTH * BLOCK_ROWS * BLOCK_HEIGHT],
        }
    }

    // Note: This is kind of hacky shortcut for picking color, not accurate NTSC emulation
    // Would be cool to revisit sometime though and really nail the NTSC artifacts
    //
    // Returns the finished frame as a flat buffer of 0x00RRGGBB pixels
    // Caller then determines how to ultimately display them
    pub(super) fn render(&mut self, frame: &FrameBuf) -> &[u32] {
        for (row, row_cells) in frame.iter().enumerate() {
            for col in 0..BLOCK_COLS {
                let colors = match row_cells[col] {
                    FrameByte::Hires(byte) => {
                        // Neighboring bytes are needed for cross-byte fringing (0 at edges)
                        let left = if col > 0 { row_cells[col - 1].val() } else { 0 };
                        let right = if col < (BLOCK_COLS - 1) {
                            row_cells[col + 1].val()
                        } else {
                            0
                        };
                        Self::map_color_hires(left, byte, right, col)
                    }
                    FrameByte::Lores(byte) => Self::map_color_lores(byte),
                    FrameByte::Text(byte) => Self::map_color_text(byte),
                };

                let base = row * (BLOCK_COLS * BLOCK_WIDTH) + col * BLOCK_WIDTH;
                for (k, &color) in colors.iter().enumerate() {
                    self.buf[base + k] = color;
                }
            }
        }

        &self.buf
    }

    fn map_color_hires(left: u8, cur: u8, right: u8, col: usize) -> [u32; BLOCK_WIDTH] {
        // The pixels for this block in order
        let mut pixel_map = [color::BLACK; BLOCK_WIDTH];

        let alt_colors = (cur >> 7) != 0; // If MSB is high, use alternate color palette

        // We need to check bordering dots, even if in adjacent bytes
        let left_block_dot = (left >> 6) & 1;
        let right_block_dot = right & 1;

        // Scan each bit (except the MSB), mapping it to a color depending on its value and its
        // neighboring bits
        let mut tmp = cur;
        for (i, pixel) in pixel_map.iter_mut().enumerate() {
            let dot = tmp & 1;
            let left_dot = if i == 0 {
                left_block_dot
            } else {
                (cur >> (i - 1)) & 1
            };
            let right_dot = if i == 6 {
                right_block_dot
            } else {
                (cur >> (i + 1)) & 1
            };

            // "Evenness" depends on block column and position within block
            let is_even = col % 2 == i % 2;

            let color = if dot != 0 {
                // Any high bit bordering another high bit becomes a white dot
                if left_dot == 1 || right_dot == 1 {
                    color::WHITE
                } else if alt_colors && is_even {
                    color::HIRES_BLUE
                } else if alt_colors && !is_even {
                    color::HIRES_ORANGE
                } else if !alt_colors && is_even {
                    color::HIRES_VIOLET
                } else {
                    color::HIRES_GREEN
                }

            // If the bit is low, but borders a high bit to its right, incorporate "fringing"
            //
            // This is not perfect, fringing is difficult to get right with its half pixel shifts
            // and whatnot
            } else if right_dot == 1 {
                if alt_colors && !is_even {
                    color::HIRES_BLUE
                } else if alt_colors && is_even {
                    color::HIRES_ORANGE
                } else if !alt_colors && !is_even {
                    color::HIRES_VIOLET
                } else {
                    color::HIRES_GREEN
                }

            // Otherwise just draw a black pixel
            } else {
                color::BLACK
            };

            tmp >>= 1;
            *pixel = color;
        }

        pixel_map
    }

    fn map_color_lores(byte: u8) -> [u32; BLOCK_WIDTH] {
        [color::LORES_MAP[(byte & 0x0F) as usize]; BLOCK_WIDTH]
    }

    fn map_color_text(byte: u8) -> [u32; BLOCK_WIDTH] {
        core::array::from_fn(|i| {
            if byte & (1 << i) != 0 {
                color::WHITE
            } else {
                color::BLACK
            }
        })
    }
}
