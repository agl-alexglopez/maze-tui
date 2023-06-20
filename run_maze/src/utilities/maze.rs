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

// Public Types

pub type Square = u16;
pub type WallLine = u16;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Point {
    pub row: i32,
    pub col: i32,
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
    pub odd_rows: i32,
    pub odd_cols: i32,
    pub style: MazeStyle,
}

#[derive(Debug, Default)]
pub struct Maze {
    maze: Vec<Vec<Square>>,
    maze_row_size: i32,
    maze_col_size: i32,
    wall_style_index: usize,
}

// Core Maze Object Implementation

impl Maze {
    pub fn new(args: MazeArgs) -> Self {
        let rows = args.odd_rows + 1 - (args.odd_rows % 2);
        let cols = args.odd_cols + 1 - (args.odd_cols % 2);
        Self {
            maze: (vec![vec![0; cols as usize]; rows as usize]),
            maze_row_size: (rows),
            maze_col_size: (cols),
            wall_style_index: (args.style as usize),
        }
    }

    pub fn row_size(&self) -> i32 {
        self.maze_row_size
    }

    pub fn col_size(&self) -> i32 {
        self.maze_col_size
    }

    pub fn wall_style(&self) -> &[&str; 16] {
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
            odd_rows: DEFAULT_ROWS,
            odd_cols: DEFAULT_COLS,
            style: MazeStyle::Sharp,
        }
    }
}

// Read Only Data Available to Any Maze Users

// Any modification made to these bits by a builder MUST be cleared before build process completes.
pub const CLEAR_AVAILABLE_BITS: Square = 0b0001_1111_1111_0000;

pub const DEFAULT_ROWS: i32 = 31;
pub const DEFAULT_COLS: i32 = 111;
pub const PATH_BIT: Square = 0b0010_0000_0000_0000;
pub const BUILDER_BIT: Square = 0b0001_0000_0000_0000;
pub const WALL_MASK: WallLine = 0b1111;
pub const FLOATING_WALL: WallLine = 0b0000;
pub const NORTH_WALL: WallLine = 0b0001;
pub const EAST_WALL: WallLine = 0b0010;
pub const SOUTH_WALL: WallLine = 0b0100;
pub const WEST_WALL: WallLine = 0b1000;
// Walls are constructed in terms of other walls they need to connect to. For example, read
// 0b0011 as, "this is a wall square that must connect to other walls to the East and North."
pub static WALL_STYLES: [[&str; 16]; 6] = [
    // 0bWestSouthEastNorth. Note: 0b0000 is a floating wall with no walls around.
    // Then, count from 0 (0b0000) to 15 (0b1111) in binary to form different wall shapes.
    [
        // sharp
        "■", "╵", "╶", "└", "╷", "│", "┌", "├", "╴", "┘", "─", "┴", "┐", "┤", "┬", "┼",
    ],
    [
        // rounded
        "●", "╵", "╶", "╰", "╷", "│", "╭", "├", "╴", "╯", "─", "┴", "╮", "┤", "┬", "┼",
    ],
    [
        // doubles
        "◫", "║", "═", "╚", "║", "║", "╔", "╠", "═", "╝", "═", "╩", "╗", "╣", "╦", "╬",
    ],
    [
        // bold
        "■", "╹", "╺", "┗", "╻", "┃", "┏", "┣", "╸", "┛", "━", "┻", "┓", "┫", "┳", "╋",
    ],
    [
        // contrast
        "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█",
    ],
    [
        // spikes
        "✸", "╀", "┾", "╊", "╁", "╂", "╆", "╊", "┽", "╃", "┿", "╇", "╅", "╉", "╈", "╋",
    ],
];

// north, east, south, west provided to any users of a maze for convenience.
pub const CARDINAL_DIRECTIONS: [Point; 4] = [
    Point { row: -1, col: 0 },
    Point { row: 0, col: 1 },
    Point { row: 1, col: 0 },
    Point { row: 0, col: -1 },
];

// south, south-east, east, north-east, north, north-west, west, south-west
pub const ALL_DIRECTIONS: [Point; 8] = [
    Point { row: 1, col: 0 },
    Point { row: 1, col: 1 },
    Point { row: 0, col: 1 },
    Point { row: -1, col: 1 },
    Point { row: -1, col: 0 },
    Point { row: -1, col: -1 },
    Point { row: 0, col: -1 },
    Point { row: 1, col: -1 },
];
