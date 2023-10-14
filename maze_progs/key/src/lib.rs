#[derive(Copy, Clone)]
pub struct ThreadColor {
    pub block: char,
    pub ansi: u8,
}

pub fn thread_color_block(color_index: usize) -> char {
    THREAD_COLORS[color_index].block
}

pub fn thread_color_code(color_index: usize) -> u8 {
    THREAD_COLORS[color_index].ansi
}

pub fn thread_color(color_index: usize) -> ThreadColor {
    THREAD_COLORS[color_index]
}

pub fn num_colors() -> usize {
    THREAD_COLORS.len()
}

// The first four colors are the thread primitives that mix to form the rest.
pub const BLOCK: char = '█';
pub const ANSI_RED: u8 = 1;
pub const ANSI_GRN: u8 = 2;
pub const ANSI_BLU: u8 = 4;
pub const ANSI_PRP: u8 = 183;
pub const ANSI_CYN: u8 = 14;
pub const ANSI_RED_BLOCK: u8 = 1;
pub const ANSI_GRN_BLOCK: u8 = 2;
pub const ANSI_YEL_BLOCK: u8 = 3;
pub const ANSI_BLU_BLOCK: u8 = 4;
pub const ANSI_PRP_BLOCK: u8 = 183;
pub const ANSI_MAG_BLOCK: u8 = 201;
pub const ANSI_CYN_BLOCK: u8 = 87;
pub const ANSI_WIT_BLOCK: u8 = 231;
pub const ANSI_PRP_RED_BLOCK: u8 = 204;
pub const ANSI_RED_GRN_BLU_BLOCK: u8 = 121;
pub const ANSI_GRN_PRP_BLOCK: u8 = 106;
pub const ANSI_GRN_BLU_PRP_BLOCK: u8 = 60;
pub const ANSI_RED_GRN_PRP_BLOCK: u8 = 105;
pub const ANSI_RED_BLU_PRP_BLOCK: u8 = 89;
pub const ANSI_DRK_BLU_MAG_BLOCK: u8 = 57;

// Threads Overlaps. The zero thread is the zero index bit with a value of 1.
// Each of the four threads has a one bit that represents them. When they
// overlap during a solution their bits mix to form interesting colors.
pub static THREAD_COLORS: [ThreadColor; 16] = [
    // 0b0000
    ThreadColor {
        block: ' ',
        ansi: 0,
    },
    // 0b0001
    ThreadColor {
        block: BLOCK,
        ansi: ANSI_RED,
    },
    // 0b0010
    ThreadColor {
        block: BLOCK,
        ansi: ANSI_GRN,
    },
    // 0b0011
    ThreadColor {
        block: BLOCK,
        ansi: 3,
    },
    // 0b0100
    ThreadColor {
        block: BLOCK,
        ansi: ANSI_BLU,
    },
    // 0b0101
    ThreadColor {
        block: BLOCK,
        ansi: 201,
    },
    // 0b0110
    ThreadColor {
        block: BLOCK,
        ansi: ANSI_CYN,
    },
    // 0b0111
    ThreadColor {
        block: BLOCK,
        ansi: 121,
    },
    // 0b1000
    ThreadColor {
        block: BLOCK,
        ansi: ANSI_PRP,
    },
    // 0b1001
    ThreadColor {
        block: BLOCK,
        ansi: 204,
    },
    // 0b1010
    ThreadColor {
        block: BLOCK,
        ansi: 106,
    },
    // 0b1011
    ThreadColor {
        block: BLOCK,
        ansi: 105,
    },
    // 0b1100
    ThreadColor {
        block: BLOCK,
        ansi: 57,
    },
    // 0b1101
    ThreadColor {
        block: BLOCK,
        ansi: 89,
    },
    // 0b1110
    ThreadColor {
        block: BLOCK,
        ansi: 60,
    },
    // 0b1111
    ThreadColor {
        block: BLOCK,
        ansi: 231,
    },
];
