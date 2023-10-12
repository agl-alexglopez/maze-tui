pub type SpeedUnit = u64;

pub struct Rgb {
    pub ch: [u8; 3],
}

pub const PAINT: maze::Square = 0b1_0000_0000;
pub const MEASURE: maze::Square = 0b10_0000_0000;
pub const NUM_PAINTERS: usize = 4;
pub const ANIMATION_SPEEDS: [SpeedUnit; 8] = [0, 10000, 5000, 2000, 1000, 500, 250, 50];

const RGB_ESCAPE: &str = "\x1b[38;2;";
const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

pub fn print_rgb(rgb: Rgb, p: maze::Point, offset: maze::Offset) {
    print::set_cursor_position(p, offset);
    print!(
        "{}",
        RGB_ESCAPE.to_owned()
            + &rgb.ch[R].to_string()
            + ";"
            + &rgb.ch[G].to_string()
            + ";"
            + &rgb.ch[B].to_string()
            + "m"
            + "█"
            + "\x1b[0m"
    );
}

pub fn animate_rgb(rgb: Rgb, p: maze::Point, offset: maze::Offset) {
    print::set_cursor_position(p, offset);
    print!(
        "{}",
        RGB_ESCAPE.to_owned()
            + &rgb.ch[R].to_string()
            + ";"
            + &rgb.ch[G].to_string()
            + ";"
            + &rgb.ch[B].to_string()
            + "m"
            + "█"
            + "\x1b[0m"
    );
    print::flush();
}
