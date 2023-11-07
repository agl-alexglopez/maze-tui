use crossterm::{
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use print::maze_panic;
use std::io::{self};

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

const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

#[inline]
pub fn has_paint_vals(square: maze::Square) -> bool {
    (square & PAINT_MASK) != 0
}

#[inline]
pub fn is_measured(square: maze::Square) -> bool {
    (square & MEASURED) != 0
}

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

pub fn animate_mini_rgb(
    rgb_top: Option<Rgb>,
    rgb_bottom: Option<Rgb>,
    p: maze::Point,
    offset: maze::Offset,
) {
    print::set_cursor_position(
        maze::Point {
            row: p.row / 2,
            col: p.col,
        },
        offset,
    );
    match (rgb_top, rgb_bottom) {
        (Some(path_above), Some(path_below)) => {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Rgb {
                    r: path_above[R],
                    g: path_above[G],
                    b: path_above[B]
                }),
                SetBackgroundColor(Color::Rgb {
                    r: path_below[R],
                    g: path_below[G],
                    b: path_below[B]
                }),
                Print('▀'),
                ResetColor,
            )
            .expect("Printer broke.");
        }
        (Some(path_with_wall_below), None) => {
            execute!(
                io::stdout(),
                SetBackgroundColor(Color::Rgb {
                    r: path_with_wall_below[R],
                    g: path_with_wall_below[G],
                    b: path_with_wall_below[B]
                }),
                Print('▄'),
                ResetColor,
            )
            .expect("Printer broke.");
        }
        (None, Some(path_with_wall_above)) => {
            execute!(
                io::stdout(),
                SetBackgroundColor(Color::Rgb {
                    r: path_with_wall_above[R],
                    g: path_with_wall_above[G],
                    b: path_with_wall_above[B]
                }),
                Print('▀'),
                ResetColor,
            )
            .expect("Printer broke.");
        }
        _ => {}
    }
}

pub fn print_mini_rgb(
    rgb_top: Option<Rgb>,
    rgb_bottom: Option<Rgb>,
    p: maze::Point,
    offset: maze::Offset,
) {
    print::set_cursor_position(
        maze::Point {
            row: p.row / 2,
            col: p.col,
        },
        offset,
    );
    match (rgb_top, rgb_bottom) {
        (Some(path_above), Some(path_below)) => {
            queue!(
                io::stdout(),
                SetForegroundColor(Color::Rgb {
                    r: path_above[R],
                    g: path_above[G],
                    b: path_above[B]
                }),
                SetBackgroundColor(Color::Rgb {
                    r: path_below[R],
                    g: path_below[G],
                    b: path_below[B]
                }),
                Print('▀'),
                ResetColor,
            )
            .expect("Printer broke.");
        }
        (Some(path_with_wall_below), None) => {
            queue!(
                io::stdout(),
                SetBackgroundColor(Color::Rgb {
                    r: path_with_wall_below[R],
                    g: path_with_wall_below[G],
                    b: path_with_wall_below[B]
                }),
                Print('▄'),
                ResetColor,
            )
            .expect("Printer broke.");
        }
        (None, Some(path_with_wall_above)) => {
            queue!(
                io::stdout(),
                SetBackgroundColor(Color::Rgb {
                    r: path_with_wall_above[R],
                    g: path_with_wall_above[G],
                    b: path_with_wall_above[B]
                }),
                Print('▀'),
                ResetColor,
            )
            .expect("Printer broke.");
        }
        _ => {}
    }
}
