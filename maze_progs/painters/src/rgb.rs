use crossterm::{
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use print::maze_panic;
use std::io::{self};

pub struct ThreadGuide {
    pub bias: usize,
    pub color_i: usize,
    pub p: maze::Point,
}

pub type SpeedUnit = u64;

pub type Rgb = [u8; 3];

pub const PAINT: maze::Square = 0b1_0000_0000;
pub const MEASURE: maze::Square = 0b10_0000_0000;
pub const NUM_PAINTERS: usize = 4;
pub const ANIMATION_SPEEDS: [SpeedUnit; 8] = [0, 10000, 5000, 2000, 1000, 500, 250, 50];

const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

pub fn print_rgb(rgb: Rgb, p: maze::Point, offset: maze::Offset) {
    print::set_cursor_position(p, offset);
    match queue!(
        io::stdout(),
        SetForegroundColor(Color::Rgb {
            r: rgb[R],
            g: rgb[G],
            b: rgb[B]
        }),
        Print('█'),
        ResetColor,
    ) {
        Ok(_) => {}
        Err(_) => maze_panic!("Could not print rgb."),
    }
}

pub fn animate_rgb(rgb: Rgb, p: maze::Point, offset: maze::Offset) {
    print::set_cursor_position(p, offset);
    match execute!(
        io::stdout(),
        SetForegroundColor(Color::Rgb {
            r: rgb[R],
            g: rgb[G],
            b: rgb[B]
        }),
        Print('█'),
        ResetColor,
    ) {
        Ok(_) => {}
        Err(_) => maze_panic!("Could not print rgb."),
    }
}
