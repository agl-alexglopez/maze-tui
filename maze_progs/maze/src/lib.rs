// Here is the scheme we will use to store tons of data in a square.
//
// When building the maze here is how we will use the available bits.
//
// We may use some color mixing bits for backtracking info.
// However, they must be cleared if used before algorithm finishes.
// 24 bit thread color mixing-----|---------------------------|
// ------------------------------ |||| |||| |||| |||| |||| ||||
// walls / thread cache------|||| |||| |||| |||| |||| |||| ||||
// --------------------------|||| |||| |||| |||| |||| |||| ||||
// maze build bit----------| |||| |||| |||| |||| |||| |||| ||||
// maze paths bit---------|| |||| |||| |||| |||| |||| |||| ||||
// maze start bit--------||| |||| |||| |||| |||| |||| |||| ||||
// maze goals bit-------|||| |||| |||| |||| |||| |||| |||| ||||
//                    0b0000 0000 0000 0000 0000 0000 0000 0000
//
// The maze builder is responsible for zeroing out the direction bits as part of the
// building process. When solving the maze we adjust how we use the middle bits.
//
// 24 bit thread color mixing-----|---------------------------|
// ------------------------------ |||| |||| |||| |||| |||| ||||
// walls / thread cache------|||| |||| |||| |||| |||| |||| ||||
// --------------------------|||| |||| |||| |||| |||| |||| ||||
// maze build bit----------| |||| |||| |||| |||| |||| |||| ||||
// maze paths bit---------|| |||| |||| |||| |||| |||| |||| ||||
// maze start bit--------||| |||| |||| |||| |||| |||| |||| ||||
// maze goals bit-------|||| |||| |||| |||| |||| |||| |||| ||||
//                    0b0000 0000 0000 0000 0000 0000 0000 0000
use std::ops::{Index, IndexMut};

// Public Types

pub type Square = u32;
pub type WallLine = u32;

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

// This is at the core of our maze. The fundamental information and structure we need.
#[derive(Debug, Clone, Default)]
pub struct Blueprint {
    pub buf: Vec<Square>,
    pub rows: i32,
    pub cols: i32,
    pub offset: Offset,
    pub wall_style_index: usize,
}

// We will also be tracking how our maze changes for the TUI animation playback.
#[derive(Debug, Default, Clone, Copy)]
pub struct Delta {
    pub id: Point,
    pub before: Square,
    pub after: Square,
    // We can enter deltas into our Tape but then specify how many squares to change in one frame.
    pub burst: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Tape {
    steps: Vec<Delta>,
    i: usize,
}

// Our maze now comes with the ability to track the history of algorithms that made and solved it.
#[derive(Debug, Clone, Default)]
pub struct Maze {
    pub maze: Blueprint,
    pub build_history: Tape,
    pub solve_history: Tape,
}
// Read Only Data Available to Any Maze Users

// Any modification made to these bits by a builder MUST be cleared before build process completes.
pub const CLEAR_AVAILABLE_BITS: Square = 0x10FFFFFF;

pub const DEFAULT_ROWS: i32 = 31;
pub const DEFAULT_COLS: i32 = 111;
pub const PATH_BIT: Square = 0x20000000;
pub const WALL_MASK: WallLine = 0xF000000;
pub const WALL_SHIFT: usize = 24;
pub const FLOATING_WALL: WallLine = 0b0;
pub const NORTH_WALL: WallLine = 0x1000000;
pub const EAST_WALL: WallLine = 0x2000000;
pub const SOUTH_WALL: WallLine = 0x4000000;
pub const WEST_WALL: WallLine = 0x8000000;
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
            build_history: Tape::default(),
            solve_history: Tape::default(),
        }
    }

    #[inline]
    pub fn rows(&self) -> i32 {
        self.maze.rows
    }

    #[inline]
    pub fn offset(&self) -> Offset {
        self.maze.offset
    }

    #[inline]
    pub fn cols(&self) -> i32 {
        self.maze.cols
    }

    #[inline]
    pub fn wall_char(&self, square: Square) -> char {
        WALL_STYLES[(self.maze.wall_style_index * WALL_ROW)
            + ((square & WALL_MASK) >> WALL_SHIFT) as usize]
    }

    #[inline]
    pub fn wall_row(&self) -> &[char] {
        &WALL_STYLES[self.maze.wall_style_index * WALL_ROW
            ..self.maze.wall_style_index * WALL_ROW + WALL_ROW]
    }

    #[inline]
    pub fn style_index(&self) -> usize {
        self.maze.wall_style_index
    }

    #[inline]
    pub fn is_mini(&self) -> bool {
        self.maze.is_mini()
    }

    #[inline]
    pub fn as_slice(&self) -> &[Square] {
        self.maze.buf.as_slice()
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [Square] {
        self.maze.buf.as_mut_slice()
    }

    #[inline]
    pub fn get_mut(&mut self, row: i32, col: i32) -> &mut Square {
        self.maze.get_mut(row, col)
    }

    #[inline]
    pub fn get(&self, row: i32, col: i32) -> Square {
        self.maze.get(row, col)
    }

    #[inline]
    pub fn wall_at(&self, row: i32, col: i32) -> bool {
        self.maze.wall_at(row, col)
    }

    #[inline]
    pub fn path_at(&self, row: i32, col: i32) -> bool {
        self.maze.path_at(row, col)
    }
}

impl Blueprint {
    #[inline]
    pub fn get(&self, row: i32, col: i32) -> Square {
        self.buf[(row * self.cols + col) as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, row: i32, col: i32) -> &mut Square {
        &mut self.buf[(row * self.cols + col) as usize]
    }

    #[inline]
    pub fn wall_char(&self, square: Square) -> char {
        WALL_STYLES
            [(self.wall_style_index * WALL_ROW) + ((square & WALL_MASK) >> WALL_SHIFT) as usize]
    }

    #[inline]
    pub fn wall_row(&self) -> &[char] {
        &WALL_STYLES[self.wall_style_index * WALL_ROW..self.wall_style_index * WALL_ROW + WALL_ROW]
    }

    #[inline]
    pub fn wall_at(&self, row: i32, col: i32) -> bool {
        (self.buf[(row * self.cols + col) as usize] & PATH_BIT) == 0
    }

    #[inline]
    pub fn path_at(&self, row: i32, col: i32) -> bool {
        (self.buf[(row * self.cols + col) as usize] & PATH_BIT) != 0
    }

    #[inline]
    pub fn is_mini(&self) -> bool {
        self.wall_style_index == (MazeStyle::Mini as usize)
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

///
/// The Tape data structure implementation is concerned with sensible ways to step through the
/// history of deltas as a maze build and solve operation completes. We only need an index and
/// we track deltas as simple before and after u32's and what square changed.
///

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

impl Tape {
    pub fn slice(&self, start: usize, end: usize) -> &[Delta] {
        &self.steps[start..end]
    }

    pub fn slice_mut(&mut self, start: usize, end: usize) -> &mut [Delta] {
        &mut self.steps[start..end]
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn end(&mut self) {
        if self.steps.is_empty() {
            panic!("no tape to end because no deltas provided");
        }
        self.i = self.steps.len() - 1;
    }

    pub fn start(&mut self) {
        if self.steps.is_empty() {
            panic!("no tape to start because no deltas provided");
        }
        self.i = 0;
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
            || self.i.overflowing_sub(self.steps[self.i - 1].burst).1
        {
            return self.i;
        }
        self.i - self.steps[self.i - 1].burst
    }

    pub fn peek_next_delta(&self) -> Option<&[Delta]> {
        if self.i + self.steps[self.i].burst >= self.steps.len() {
            return None;
        }
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn peek_prev_delta(&self) -> Option<&[Delta]> {
        if self.i == 0 || self.i.overflowing_sub(self.steps[self.i - 1].burst).1 {
            return None;
        }
        Some(&self.steps[self.i - self.steps[self.i - 1].burst..self.i])
    }

    pub fn next_delta(&mut self) -> Option<&[Delta]> {
        if self.steps.is_empty() || self.i + self.steps[self.i].burst >= self.steps.len() {
            return None;
        }
        self.i += self.steps[self.i].burst;
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn prev_delta(&mut self) -> Option<&[Delta]> {
        if self.i == 0 || self.i.overflowing_sub(self.steps[self.i - 1].burst).1 {
            return None;
        }
        self.i -= self.steps[self.i - 1].burst;
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn push_burst(&mut self, steps: &[Delta]) {
        if !steps.is_empty()
            && (steps[0].burst != steps.len() || steps[steps.len() - 1].burst != steps.len())
        {
            panic!(
                "ill formed burst input burst burst[0]={},burst[burst-1]={}, len {}",
                steps[0].burst,
                steps[steps.len() - 1].burst,
                steps.len()
            );
        }
        for s in steps.iter() {
            self.steps.push(*s);
        }
    }

    pub fn push(&mut self, s: Delta) {
        self.steps.push(s);
    }

    pub fn at_end(&self) -> bool {
        self.i == self.peek_next_index()
    }

    pub fn at_start(&self) -> bool {
        self.i == self.peek_prev_index()
    }

    pub fn set_prev(&mut self) -> bool {
        let prev = self.i;
        self.i = self.peek_prev_index();
        self.i != prev
    }

    pub fn set_next(&mut self) -> bool {
        let prev = self.i;
        self.i = self.peek_next_index();
        self.i != prev
    }
}

///
/// Free functions for when the maze object is not present but we still want static info.
///

#[inline]
pub fn wall_row(row_index: usize) -> &'static [char] {
    &WALL_STYLES[row_index * WALL_ROW..row_index * WALL_ROW + WALL_ROW]
}

#[inline]
pub fn wall_char(style_index: usize, square: Square) -> char {
    WALL_STYLES[style_index * WALL_ROW + ((square & WALL_MASK) >> WALL_SHIFT) as usize]
}

#[inline]
pub fn is_wall(square: Square) -> bool {
    (square & PATH_BIT) == 0
}

#[inline]
pub fn is_path(square: Square) -> bool {
    (square & PATH_BIT) != 0
}
