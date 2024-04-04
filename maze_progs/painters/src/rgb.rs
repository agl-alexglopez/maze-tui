pub struct ThreadGuide {
    pub bias: usize,
    pub color_i: usize,
    pub cache: maze::Square,
    pub p: maze::Point,
}

pub type SpeedUnit = u64;

pub type Rgb = [u8; 3];

pub const PAINT: maze::Square = 0b0001_0000_0000_0000_0000_0000_0000;
pub const PAINT_MASK: maze::Square = 0b1111_1111_1111_1111_1111_1111;
pub const MEASURED: maze::Square = 0b0010_0000_0000_0000_0000_0000_0000;
pub const MEASURED_MASKS: [maze::Square; 4] = [0x1000000, 0x2000000, 0x4000000, 0x8000000];
pub const NUM_PAINTERS: usize = 4;
pub const ANIMATION_SPEEDS: [SpeedUnit; 8] = [0, 10000, 5000, 2000, 1000, 500, 250, 50];
pub const RED_SHIFT: maze::Square = 16;
pub const GREEN_SHIFT: maze::Square = 8;

#[inline]
pub fn has_paint_vals(square: maze::Square) -> bool {
    (square & PAINT_MASK) != 0
}

#[inline]
pub fn is_measured(square: maze::Square) -> bool {
    (square & MEASURED) != 0
}
