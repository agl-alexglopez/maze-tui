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

#[derive(Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Delta {
    pub p: Point,
    pub before: Square,
    pub after: Square,
    pub burst: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Tape {
    steps: Vec<Delta>,
    i: usize,
}

impl Tape {
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn cur_step(&self) -> Option<&[Delta]> {
        if self.steps.is_empty() {
            return None;
        }
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    fn peek_next_index(&self) -> usize {
        if self.steps.is_empty() || self.i + self.steps[self.i].burst >= self.steps.len() {
            return self.i;
        }
        self.i + self.steps[self.i].burst
    }

    fn peek_prev_index(&self) -> usize {
        if self.steps.is_empty()
            || self.i == 0
            || self.i.overflowing_sub(self.i - self.steps[self.i].burst).1
        {
            return self.i;
        }
        self.i - self.steps[self.i].burst
    }

    pub fn peek_next_delta(&self) -> Option<&[Delta]> {
        if self.i + self.steps[self.i].burst >= self.steps.len() {
            return None;
        }
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn peek_prev_delta(&self) -> Option<&[Delta]> {
        if self.i == 0 || self.i.overflowing_sub(self.steps[self.i].burst).1 {
            return None;
        }
        Some(&self.steps[self.i - self.steps[self.i].burst..self.i])
    }

    pub fn next_delta(&mut self) -> Option<&[Delta]> {
        if self.steps.is_empty() || self.i + self.steps[self.i].burst >= self.steps.len() {
            return None;
        }
        self.i += self.steps[self.i].burst;
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn prev_delta(&mut self) -> Option<&[Delta]> {
        let (_, overflowed) = self.i.overflowing_sub(self.steps[self.i].burst);
        if self.i == 0 || overflowed {
            return None;
        }
        self.i -= self.steps[self.i].burst;
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn push_burst(&mut self, steps: &[Delta]) {
        if steps.is_empty()
            || steps[0].burst != steps.len()
            || steps[steps.len() - 1].burst != steps.len()
        {
            panic!("ill formed burst input burst, specified burst: {:?}", steps);
        }
        for s in steps.iter() {
            self.steps.push(*s);
        }
    }

    pub fn push(&mut self, s: Delta) {
        if s.burst != 1 {
            panic!("single delta has burst length of {}", s.burst);
        }
        self.steps.push(s);
    }

    pub fn at_end(&self) -> bool {
        self.i == self.peek_next_index()
    }

    pub fn at_start(&self) -> bool {
        self.i == self.peek_prev_index()
    }

    pub fn move_tape_prev(&mut self) -> bool {
        let prev = self.i;
        self.i = self.peek_prev_index();
        self.i != prev
    }

    pub fn move_tape_next(&mut self) -> bool {
        let prev = self.i;
        self.i = self.peek_next_index();
        self.i != prev
    }
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
    pub build_history: Tape,
    pub solve_history: Tape,
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
            wall_style_index: args.style as usize,
            receiver: None,
            build_history: Tape::default(),
            solve_history: Tape::default(),
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
            wall_style_index: args.style as usize,
            receiver: Some(rec),
            build_history: Tape::default(),
            solve_history: Tape::default(),
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

    pub fn wall_char(&self, wall_mask_index: usize) -> char {
        WALL_STYLES[(self.wall_style_index * WALL_ROW) + wall_mask_index]
    }

    pub fn style_index(&self) -> usize {
        self.wall_style_index
    }

    pub fn is_mini(&self) -> bool {
        self.wall_style_index == (MazeStyle::Mini as usize)
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

impl Index<usize> for Tape {
    type Output = Delta;
    fn index(&self, index: usize) -> &Self::Output {
        &self.steps[index]
    }
}

impl IndexMut<usize> for Tape {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.steps[index]
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
