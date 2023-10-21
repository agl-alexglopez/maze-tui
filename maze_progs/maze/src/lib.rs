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
use crossbeam_channel::Receiver;
use std::ops::{Index, IndexMut};

// Public Types

pub type Square = u16;
pub type WallLine = u16;

#[derive(Default, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Point {
    pub row: i32,
    pub col: i32,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Offset {
    pub add_rows: i32,
    pub add_cols: i32,
}

#[derive(Clone, Copy)]
pub enum MazeStyle {
    Sharp = 0,
    Round,
    Doubles,
    Bold,
    Contrast,
    Half,
    Spikes,
}

#[derive(Clone, Copy)]
pub struct MazeArgs {
    pub odd_rows: i32,
    pub odd_cols: i32,
    pub offset: Offset,
    pub style: MazeStyle,
}

// Model a ROWxCOLUMN maze in a flat Vec. Implement tricky indexing in Index impls.
#[derive(Debug, Clone)]
pub struct Maze {
    maze: Vec<Square>,
    maze_row_size: i32,
    maze_col_size: i32,
    offset: Offset,
    wall_style_index: usize,
    receiver: Option<Receiver<bool>>,
}

// Core Maze Object Implementation

// A maze in this program is intended to be shared both mutably and immutably.
// A maze only provides building blocks and some convenience read-only data.
// Builders and solvers use the visitor pattern to operate on and extend
// what they wish on the maze.
impl Maze {
    pub fn new(args: MazeArgs) -> Self {
        let rows = args.odd_rows + 1 - (args.odd_rows % 2);
        let cols = args.odd_cols + 1 - (args.odd_cols % 2);
        Self {
            maze: (vec![0; rows as usize * cols as usize]),
            maze_row_size: (rows),
            maze_col_size: (cols),
            offset: args.offset,
            wall_style_index: (args.style as usize),
            receiver: None,
        }
    }

    pub fn new_channel(args: &MazeArgs, rec: Receiver<bool>) -> Self {
        let rows = args.odd_rows + 1 - (args.odd_rows % 2);
        let cols = args.odd_cols + 1 - (args.odd_cols % 2);
        Self {
            maze: (vec![0; rows as usize * cols as usize]),
            maze_row_size: (rows),
            maze_col_size: (cols),
            offset: args.offset,
            wall_style_index: (args.style as usize),
            receiver: Some(rec),
        }
    }

    pub fn exit(&self) -> bool {
        match &self.receiver {
            Some(rec) => rec.is_full(),
            None => false,
        }
    }

    pub fn row_size(&self) -> i32 {
        self.maze_row_size
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn col_size(&self) -> i32 {
        self.maze_col_size
    }

    pub fn wall_style(&self) -> &[&'static str] {
        &WALL_STYLES
            [(self.wall_style_index * WALL_ROW)..(self.wall_style_index * WALL_ROW + WALL_ROW)]
    }

    pub fn style_index(&self) -> usize {
        self.wall_style_index
    }
}

impl Index<usize> for Maze {
    type Output = [Square];
    fn index(&self, index: usize) -> &Self::Output {
        &self.maze[(index * self.maze_col_size as usize)
            ..(index * self.maze_col_size as usize + self.maze_col_size as usize)]
    }
}

impl IndexMut<usize> for Maze {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.maze[(index * self.maze_col_size as usize)
            ..(index * self.maze_col_size as usize + self.maze_col_size as usize)]
    }
}

impl Default for MazeArgs {
    fn default() -> Self {
        Self {
            odd_rows: DEFAULT_ROWS,
            odd_cols: DEFAULT_COLS,
            style: MazeStyle::Sharp,
            offset: Offset::default(),
        }
    }
}

// Read Only Data Available to Any Maze Users

// Any modification made to these bits by a builder MUST be cleared before build process completes.
pub const CLEAR_AVAILABLE_BITS: Square = 0b0001_1111_1111_0000;

pub const DEFAULT_ROWS: i32 = 31;
pub const DEFAULT_COLS: i32 = 111;
pub const PATH_BIT: Square = 0b0010_0000_0000_0000;
pub const WALL_MASK: WallLine = 0b1111;
pub const FLOATING_WALL: WallLine = 0b0000;
pub const NORTH_WALL: WallLine = 0b0001;
pub const EAST_WALL: WallLine = 0b0010;
pub const SOUTH_WALL: WallLine = 0b0100;
pub const WEST_WALL: WallLine = 0b1000;
// Walls are constructed in terms of other walls they need to connect to. For example, read
// 0b0011 as, "this is a wall square that must connect to other walls to the East and North."
const WALL_ROW: usize = 16;
pub static WALL_STYLES: [&str; 112] = [
    // 0bWestSouthEastNorth. Note: 0b0000 is a floating wall with no walls around.
    // Then, count from 0 (0b0000) to 15 (0b1111) in binary to form different wall shapes.
    // sharp
    "■", "╵", "╶", "└", "╷", "│", "┌", "├", "╴", "┘", "─", "┴", "┐", "┤", "┬", "┼",
    // rounded
    "●", "╵", "╶", "╰", "╷", "│", "╭", "├", "╴", "╯", "─", "┴", "╮", "┤", "┬", "┼",
    // doubles
    "◫", "║", "═", "╚", "║", "║", "╔", "╠", "═", "╝", "═", "╩", "╗", "╣", "╦", "╬",
    // bold
    "■", "╹", "╺", "┗", "╻", "┃", "┏", "┣", "╸", "┛", "━", "┻", "┓", "┫", "┳", "╋",
    // contrast
    "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█", "█",
    // half
    "▄", "▀", "▄", "█", "▄", "█", "▄", "█", "▄", "█", "▄", "█", "▄", "█", "▄", "█",
    // spikes
    "✸", "╀", "┾", "╊", "╁", "╂", "╆", "╊", "┽", "╃", "┿", "╇", "╅", "╉", "╈", "╋",
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
