pub struct ThreadColor {
    pub block: &'static str,
    pub binary: &'static str,
    pub code: u8,
}

// The first four colors are the thread primitives that mix to form the rest.
pub const ANSI_RED: u8 = 1;
pub const ANSI_GRN: u8 = 2;
pub const ANSI_BLU: u8 = 4;
pub const ANSI_PRP: u8 = 183;
pub const ANSI_CYN: u8 = 14;
pub const ANSI_BLOCK: &str = "█";
pub const ANSI_RED_BLOCK: &str = "\x1b[38;5;1m█\x1b[0m";
pub const ANSI_GRN_BLOCK: &str = "\x1b[38;5;2m█\x1b[0m";
pub const ANSI_YEL_BLOCK: &str = "\x1b[38;5;3m█\x1b[0m";
pub const ANSI_BLU_BLOCK: &str = "\x1b[38;5;4m█\x1b[0m";
pub const ANSI_PRP_BLOCK: &str = "\x1b[38;5;183m█\x1b[0m";
pub const ANSI_MAG_BLOCK: &str = "\x1b[38;5;201m█\x1b[0m";
pub const ANSI_CYN_BLOCK: &str = "\x1b[38;5;87m█\x1b[0m";
pub const ANSI_WIT_BLOCK: &str = "\x1b[38;5;231m█\x1b[0m";
pub const ANSI_PRP_RED_BLOCK: &str = "\x1b[38;5;204m█\x1b[0m";
pub const ANSI_RED_GRN_BLU_BLOCK: &str = "\x1b[38;5;121m█\x1b[0m";
pub const ANSI_GRN_PRP_BLOCK: &str = "\x1b[38;5;106m█\x1b[0m";
pub const ANSI_GRN_BLU_PRP_BLOCK: &str = "\x1b[38;5;60m█\x1b[0m";
pub const ANSI_RED_GRN_PRP_BLOCK: &str = "\x1b[38;5;105m█\x1b[0m";
pub const ANSI_RED_BLU_PRP_BLOCK: &str = "\x1b[38;5;89m█\x1b[0m";
pub const ANSI_DRK_BLU_MAG_BLOCK: &str = "\x1b[38;5;57m█\x1b[0m";
pub const ANSI_START: &str = "\x1b[1m\x1b[38;5;87mS\x1b[0m";
// Threads Overlaps. The zero thread is the zero index bit with a value of 1.
// Each of the four threads has a one bit that represents them. When they
// overlap during a solution their bits mix to form interesting colors.
pub static THREAD_COLORS: [ThreadColor; 16] = [
    ThreadColor {
        block: "?",
        binary: "0b0000",
        code: 0,
    },
    ThreadColor {
        block: ANSI_RED_BLOCK,
        binary: "0b0001",
        code: ANSI_RED,
    },
    ThreadColor {
        block: ANSI_GRN_BLOCK,
        binary: "0b0010",
        code: ANSI_GRN,
    },
    ThreadColor {
        block: ANSI_YEL_BLOCK,
        binary: "0b0011",
        code: 3,
    },
    ThreadColor {
        block: ANSI_BLU_BLOCK,
        binary: "0b0100",
        code: ANSI_BLU,
    },
    ThreadColor {
        block: ANSI_MAG_BLOCK,
        binary: "0b0101",
        code: 201,
    },
    ThreadColor {
        block: ANSI_CYN_BLOCK,
        binary: "0b0110",
        code: ANSI_CYN,
    },
    ThreadColor {
        block: ANSI_RED_GRN_BLU_BLOCK,
        binary: "0b0111",
        code: 121,
    },
    ThreadColor {
        block: ANSI_PRP_BLOCK,
        binary: "0b1000",
        code: ANSI_PRP,
    },
    ThreadColor {
        block: ANSI_PRP_RED_BLOCK,
        binary: "0b1001",
        code: 204,
    },
    ThreadColor {
        block: ANSI_GRN_PRP_BLOCK,
        binary: "0b1010",
        code: 106,
    },
    ThreadColor {
        block: ANSI_RED_GRN_PRP_BLOCK,
        binary: "0b1011",
        code: 105,
    },
    ThreadColor {
        block: ANSI_DRK_BLU_MAG_BLOCK,
        binary: "0b1100",
        code: 57,
    },
    ThreadColor {
        block: ANSI_RED_BLU_PRP_BLOCK,
        binary: "0b1101",
        code: 89,
    },
    ThreadColor {
        block: ANSI_GRN_BLU_PRP_BLOCK,
        binary: "0b1110",
        code: 60,
    },
    ThreadColor {
        block: ANSI_WIT_BLOCK,
        binary: "0b1111",
        code: 231,
    },
];
