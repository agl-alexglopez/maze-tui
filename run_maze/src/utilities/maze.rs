// Here is the scheme we will use to store tons of data in a square.
//
// When building the maze here is how we will use the available bits.
//
// wall structure----------------------||||
// ------------------------------------||||
// 0 backtrack marker bit------------| ||||
// 1 backtrack marker bit ----------|| ||||
// 2 backtrack marker bit----------||| ||||
// 3 unused-----------------------|||| ||||
// -------------------------------|||| ||||
// 0 unused bit-----------------| |||| ||||
// 1 unused bit----------------|| |||| ||||
// 2 unused bit---------------||| |||| ||||
// 3 unused bit--------------|||| |||| ||||
// --------------------------|||| |||| ||||
// maze build bit----------| |||| |||| ||||
// maze paths bit---------|| |||| |||| ||||
// maze start bit--------||| |||| |||| ||||
// maze goals bit-------|||| |||| |||| ||||
//                    0b0000 0000 0000 0000
//
// The maze builder is responsible for zeroing out the direction bits as part of the
// building process. When solving the maze we adjust how we use the middle bits.
//
// wall structure----------------------||||
// ------------------------------------||||
// 0 thread paint--------------------| ||||
// 1 thread paint-------------------|| ||||
// 2 thread paint------------------||| ||||
// 3 thread paint-----------------|||| ||||
// -------------------------------|||| ||||
// 0 thread cache---------------| |||| ||||
// 1 thread cache--------------|| |||| ||||
// 2 thread cache-------------||| |||| ||||
// 3 thread cache------------|||| |||| ||||
// --------------------------|||| |||| ||||
// maze build bit----------| |||| |||| ||||
// maze paths bit---------|| |||| |||| ||||
// maze start bit--------||| |||| |||| ||||
// maze goals bit-------|||| |||| |||| ||||
//                    0b0000 0000 0000 0000
use std::ops::{Index, IndexMut};

type Square = i16;
type WallLine = i16;
type BacktrackMarker = i16;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Point {
    row: i32,
    col: i32,
}

pub enum MazeStyle {
    Sharp = 0,
    Round,
    Doubles,
    Bold,
    Contrast,
    Spikes,
}

pub struct MazeArgs {
    odd_rows: i32,
    odd_cols: i32,
    style: MazeStyle,
}

pub static PATH_BIT: Square = 0b0010_0000_0000_0000;
pub static CLEAR_AVAILABLE_BITS: Square = 0b0001_1111_1111_0000;
pub static START_BIT: Square = 0b0100_0000_0000_0000;
pub static BUILDER_BIT: Square = 0b0001_0000_0000_0000;
pub static MARKER_SHIFT: i8 = 4;
pub static DEFAULT_ROWS: i32 = 31;
pub static DEFAULT_COLS: i32 = 111;
pub static MARKERS_MASK: BacktrackMarker = 0b1111_0000;
pub static IS_ORIGIN: BacktrackMarker = 0b0000_0000;
pub static FROM_NORTH: BacktrackMarker = 0b0001_0000;
pub static FROM_EAST: BacktrackMarker = 0b0010_0000;
pub static FROM_SOUTH: BacktrackMarker = 0b0011_0000;
pub static FROM_WEST: BacktrackMarker = 0b0100_0000;
pub static FROM_NORTH_MARK: &'static str = "\x1b[38;5;15m\x1b[48;5;1m↑\x1b[0m";
pub static FROM_EAST_MARK: &'static str = "\x1b[38;5;15m\x1b[48;5;2m→\x1b[0m";
pub static FROM_SOUTH_MARK: &'static str = "\x1b[38;5;15m\x1b[48;5;3m↓\x1b[0m";
pub static FROM_WEST_MARK: &'static str = "\x1b[38;5;15m\x1b[48;5;4m←\x1b[0m";
pub static BACKTRACKING_SYMBOLS: [&'static str; 5] = [
    " ",
    FROM_NORTH_MARK,
    FROM_EAST_MARK,
    FROM_SOUTH_MARK,
    FROM_WEST_MARK,
];
pub static BACKTRACKING_MARKS: [Point; 5] = [
    Point { row: 0, col: 0 },
    Point { row: -2, col: 0 },
    Point { row: 0, col: 2 },
    Point { row: 2, col: 0 },
    Point { row: 0, col: -2 },
];
pub static WALL_MASK: WallLine = 0b1111;
pub static FLOATING_WALL: WallLine = 0b0000;
pub static NORTH_WALL: WallLine = 0b0001;
pub static EAST_WALL: WallLine = 0b0010;
pub static SOUTH_WALL: WallLine = 0b0100;
pub static WEST_WALL: WallLine = 0b1000;
// Walls are constructed in terms of other walls they need to connect to. For example, read
// 0b0011 as, "this is a wall square that must connect to other walls to the East and North."
pub static WALL_STYLES: [[&'static str; 16]; 6] = [
    // 0bWestSouthEastNorth. Note: 0b0000 is a floating wall with no walls around.
    // 0b0000  0b0001  0b0010  0b0011  0b0100  0b0101  0b0110  0b0111
    // 0b1000  0b1001  0b1010  0b1011  0b1100  0b1101  0b1110  0b1111
    [
        "■", "╵", "╶", "└", "╷", "│", "┌", "├", "╴", "┘", "─", "┴", "┐", "┤", "┬", "┼",
    ], // standard
    [
        "●", "╵", "╶", "╰", "╷", "│", "╭", "├", "╴", "╯", "─", "┴", "╮", "┤", "┬", "┼",
    ], // rounded
    [
        "◫", "║", "═", "╚", "║", "║", "╔", "╠", "═", "╝", "═", "╩", "╗", "╣", "╦", "╬",
    ], // doubles
    [
        "■", "╹", "╺", "┗", "╻", "┃", "┏", "┣", "╸", "┛", "━", "┻", "┓", "┫", "┳", "╋",
    ], // bold
    [
        "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█",
    ], // contrast
    [
        "✸", "╀", "┾", "╊", "╁", "╂", "╆", "╊", "┽", "╃", "┿", "╇", "╅", "╉", "╈", "╋",
    ], // spikes
];
// north, east, south, west
pub static CARDINAL_DIRECTIONS: [Point; 4] = [
    Point { row: -1, col: 0 },
    Point { row: 0, col: 1 },
    Point { row: 1, col: 0 },
    Point { row: 0, col: -1 },
];
pub static GENERATE_DIRECTIONS: [Point; 4] = [
    Point { row: -2, col: 0 },
    Point { row: 0, col: 2 },
    Point { row: 2, col: 0 },
    Point { row: 0, col: -2 },
];
// south, south-east, east, north-east, north, north-west, west, south-west
pub static ALL_DIRECTIONS: [Point; 8] = [
    Point { row: 1, col: 0 },
    Point { row: 1, col: 1 },
    Point { row: 0, col: 1 },
    Point { row: -1, col: 1 },
    Point { row: -1, col: 0 },
    Point { row: -1, col: -1 },
    Point { row: 0, col: -1 },
    Point { row: 1, col: -1 },
];

#[derive(Debug, Default)]
pub struct Maze {
    maze: Vec<Vec<Square>>,
    maze_row_size: i32,
    maze_col_size: i32,
    wall_style_index: usize,
}

impl Maze {
    pub fn new(mut args: MazeArgs) -> Self {
        if args.odd_rows % 2 == 0 {
            args.odd_rows += 1;
        }
        if args.odd_cols % 2 == 0 {
            args.odd_cols += 1;
        }
        Self {
            maze: (vec![vec![0; args.odd_cols as usize]; args.odd_rows as usize]),
            maze_row_size: (args.odd_rows),
            maze_col_size: (args.odd_cols),
            wall_style_index: (args.style as usize),
        }
    }

    pub fn row_size(&self) -> i32 {
        self.maze_row_size
    }

    pub fn col_size(&self) -> i32 {
        self.maze_col_size
    }

    pub fn wall_style(&self) -> &[&'static str; 16] {
        &WALL_STYLES[self.wall_style_index]
    }
}

impl Index<usize> for Maze {
    type Output = Vec<Square>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.maze[index]
    }
}

impl IndexMut<usize> for Maze {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.maze[index]
    }
}

impl Default for MazeArgs {
    fn default() -> Self {
        Self {
            odd_rows: 31,
            odd_cols: 111,
            style: MazeStyle::Sharp,
        }
    }
}
