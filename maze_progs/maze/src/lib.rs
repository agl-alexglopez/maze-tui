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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MazeStyle {
    Mini = 0,
    Sharp,
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

#[derive(Debug, Clone, Default)]
pub struct Blueprint {
    pub buf: Vec<Square>,
    pub rows: i32,
    pub cols: i32,
    pub offset: Offset,
    pub wall_style_index: usize,
}

// Model a ROWxCOLUMN maze in a flat Vec. Implement tricky indexing in Index impls.
#[derive(Debug, Clone, Default)]
pub struct Maze {
    maze: Blueprint,
    pub build_history: tape::Tape<Point, Square>,
    pub solve_history: tape::Tape<Point, Square>,
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
            maze: Blueprint {
                buf: (vec![0; rows as usize * cols as usize]),
                rows,
                cols,
                offset: args.offset,
                wall_style_index: args.style as usize,
            },
            build_history: tape::Tape::default(),
            solve_history: tape::Tape::default(),
        }
    }

    pub fn row_size(&self) -> i32 {
        self.maze.rows
    }

    pub fn offset(&self) -> Offset {
        self.maze.offset
    }

    pub fn col_size(&self) -> i32 {
        self.maze.cols
    }

    pub fn wall_char(&self, wall_mask_index: usize) -> char {
        WALL_STYLES[(self.maze.wall_style_index * WALL_ROW) + wall_mask_index]
    }

    pub fn wall_row(&self) -> &[char] {
        &WALL_STYLES[self.maze.wall_style_index * WALL_ROW
            ..self.maze.wall_style_index * WALL_ROW + WALL_ROW]
    }

    pub fn style_index(&self) -> usize {
        self.maze.wall_style_index
    }

    pub fn is_mini(&self) -> bool {
        self.maze.wall_style_index == (MazeStyle::Mini as usize)
    }

    pub fn as_slice(&self) -> &[Square] {
        self.maze.buf.as_slice()
    }

    pub fn as_slice_mut(&mut self) -> &mut [Square] {
        self.maze.buf.as_mut_slice()
    }

    pub fn as_blueprint_mut(&mut self) -> &mut Blueprint {
        &mut self.maze
    }

    pub fn get_mut(&mut self, row: i32, col: i32) -> &mut Square {
        &mut self.maze.buf[(row * self.maze.cols + col) as usize]
    }

    pub fn get(&self, row: i32, col: i32) -> Square {
        self.maze.buf[(row * self.maze.cols + col) as usize]
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

pub fn wall_row(row_index: usize) -> &'static [char] {
    &WALL_STYLES[row_index * WALL_ROW..row_index * WALL_ROW + WALL_ROW]
}

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
pub static WALL_STYLES: [char; 128] = [
    // 0bWestSouthEastNorth. Note: 0b0000 is a floating wall with no walls around.
    // Then, count from 0 (0b0000) to 15 (0b1111) in binary to form different wall shapes.
    // mini
    '▀', '▀', '▀', '▀', '█', '█', '█', '█', '▀', '▀', '▀', '▀', '█', '█', '█', '█',
    // sharp
    '■', '╵', '╶', '└', '╷', '│', '┌', '├', '╴', '┘', '─', '┴', '┐', '┤', '┬', '┼',
    // rounded
    '●', '╵', '╶', '╰', '╷', '│', '╭', '├', '╴', '╯', '─', '┴', '╮', '┤', '┬', '┼',
    // doubles
    '◫', '║', '═', '╚', '║', '║', '╔', '╠', '═', '╝', '═', '╩', '╗', '╣', '╦', '╬',
    // bold
    '■', '╹', '╺', '┗', '╻', '┃', '┏', '┣', '╸', '┛', '━', '┻', '┓', '┫', '┳', '╋',
    // contrast
    '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█',
    // half
    '▄', '█', '▄', '█', '▄', '█', '▄', '█', '▄', '█', '▄', '█', '▄', '█', '▄', '█',
    // spikes
    '✸', '╀', '┾', '╊', '╁', '╂', '╆', '╊', '┽', '╃', '┿', '╇', '╅', '╉', '╈', '╋',
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
