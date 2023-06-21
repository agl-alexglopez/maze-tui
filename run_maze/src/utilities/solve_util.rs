use crate::maze;
use crate::print_util;

use rand::prelude::*;
use std::io::{stdout, Write};

pub type ThreadPaint = u16;
pub type ThreadCache = u16;
pub type SolveSpeedUnit = u64;

pub struct ThreadId {
    pub index: usize,
    pub paint: ThreadPaint,
}

pub enum MazeGame {
    Hunt = 0,
    Gather,
    Corners,
}

pub enum SolverSpeed {
    Instant = 0,
    Speed1,
    Speed2,
    Speed3,
    Speed4,
    Speed5,
    Speed6,
    Speed7,
}

// Public Module Functions

pub fn set_corner_starts(maze: &maze::Maze) -> [maze::Point; 4] {
    let mut point1: maze::Point = maze::Point { row: 1, col: 1 };
    if (maze[point1.row as usize][point1.col as usize] & maze::PATH_BIT) == 0 {
        point1 = find_nearest_square(maze, point1);
    }
    let mut point2: maze::Point = maze::Point {
        row: 1,
        col: maze.col_size() - 2,
    };
    if (maze[point2.row as usize][point2.col as usize] & maze::PATH_BIT) == 0 {
        point2 = find_nearest_square(maze, point2);
    }
    let mut point3: maze::Point = maze::Point {
        row: maze.row_size() - 2,
        col: 1,
    };
    if (maze[point3.row as usize][point3.col as usize] & maze::PATH_BIT) == 0 {
        point3 = find_nearest_square(maze, point3);
    }
    let mut point4: maze::Point = maze::Point {
        row: maze.row_size() - 2,
        col: maze.col_size() - 2,
    };
    if (maze[point4.row as usize][point4.col as usize] & maze::PATH_BIT) == 0 {
        point4 = find_nearest_square(maze, point4);
    }
    return [point1, point2, point3, point4];
}

pub fn pick_random_point(maze: &maze::Maze) -> maze::Point {
    let mut gen = thread_rng();
    let choice = maze::Point {
        row: gen.gen_range(1..maze.row_size() - 2),
        col: gen.gen_range(1..maze.col_size() - 2),
    };
    if is_valid_start_or_finish(maze, choice) {
        return choice;
    }
    find_nearest_square(maze, choice)
}

pub fn find_nearest_square(maze: &maze::Maze, choice: maze::Point) -> maze::Point {
    for p in &maze::ALL_DIRECTIONS {
        let next = maze::Point {
            row: choice.row + p.row,
            col: choice.col + p.col,
        };
        if is_valid_start_or_finish(maze, next) {
            return next;
        }
    }
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            let cur = maze::Point { row: r, col: c };
            if is_valid_start_or_finish(maze, cur) {
                return cur;
            }
        }
    }
    print_paths(maze);
    panic!("Could not place a point in this maze. Was it built correctly?");
}

pub fn clear_and_flush_paths(maze: &maze::Maze) {
    print_util::clear_screen();
    print_paths(maze);
}

pub fn print_paths(maze: &maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            print_point(maze, maze::Point { row: r, col: c });
        }
        print!("\n");
    }
    stdout().flush().unwrap();
}

pub fn flush_cursor_path_coordinate(maze: &maze::Maze, point: maze::Point) {
    print_util::set_cursor_position(point);
    print_point(maze, point);
    stdout().flush().unwrap();
}

pub fn print_point(maze: &maze::Maze, point: maze::Point) {
    let square = &maze[point.row as usize][point.col as usize];
    if (square & FINISH_BIT) != 0 {
        print!("{}", ANSI_FINISH);
        return;
    }
    if (square & START_BIT) != 0 {
        print!("{}", ANSI_START);
        return;
    }
    if (square & THREAD_MASK) != 0 {
        let thread_color: ThreadPaint = (square & THREAD_MASK) >> THREAD_TAG_OFFSET;
        print!("{}", THREAD_COLORS[thread_color as usize]);
        return;
    }
    if (square & maze::PATH_BIT) == 0 {
        print!("{}", maze.wall_style()[(square & maze::WALL_MASK) as usize]);
        return;
    }
    if (square & maze::PATH_BIT) != 0 {
        print!(" ");
        return;
    }
    panic!("Uncategorized maze square! Check the bits.");
}

pub fn print_hunt_solution_message(winning_index: Option<usize>) {
    if winning_index.is_none() {
        print!("{}", THREAD_COLORS[ALL_THREADS_FAILED_INDEX]);
        return;
    }
    print!(
        "{} thread won!",
        THREAD_COLORS[((THREAD_MASKS[winning_index.unwrap()]) >> THREAD_TAG_OFFSET) as usize]
    );
}

pub fn print_gather_solution_message() {
    for mask in &THREAD_MASKS {
        print!("{}", THREAD_COLORS[(mask >> THREAD_TAG_OFFSET) as usize]);
    }
    print!(" All threads found their finish squares!\n");
}

pub fn print_overlap_key() {
    print!("┌────────────────────────────────────────────────────────────────┐\n");
    print!("│     Overlap Key: 3_THREAD | 2_THREAD | 1_THREAD | 0_THREAD     │\n");
    print!("├────────────┬────────────┬────────────┬────────────┬────────────┤\n");
    print!(
        "│ {} = 0      │ {} = 1      │ {} = 1|0    │ {} = 2      │ {} = 2|0    │\n",
        THREAD_COLORS[1], THREAD_COLORS[2], THREAD_COLORS[3], THREAD_COLORS[4], THREAD_COLORS[5]
    );
    print!("├────────────┼────────────┼────────────┼────────────┼────────────┤\n");
    print!(
        "│ {} = 2|1    │ {} = 2|1|0  │ {} = 3      │ {} = 3|0    │ {} = 3|1    │\n",
        THREAD_COLORS[6], THREAD_COLORS[7], THREAD_COLORS[8], THREAD_COLORS[9], THREAD_COLORS[10]
    );
    print!("├────────────┼────────────┼────────────┼────────────┼────────────┤\n");
    print!(
        "│ {} = 3|1|0  │ {} = 3|2    │ {} = 3|2|0  │ {} = 3|2|1  │ {} = 3|2|1|0│\n",
        THREAD_COLORS[11],
        THREAD_COLORS[12],
        THREAD_COLORS[13],
        THREAD_COLORS[14],
        THREAD_COLORS[15]
    );
    print!("└────────────┴────────────┴────────────┴────────────┴────────────┘\n");
}

// Private Module Function

fn is_valid_start_or_finish(maze: &maze::Maze, choice: maze::Point) -> bool {
    return choice.row > 0
        && choice.row < maze.row_size() - 1
        && choice.col > 0
        && choice.col < maze.col_size() - 1
        && (maze[choice.row as usize][choice.col as usize] & maze::BUILDER_BIT) != 0;
}

// Read Only Data Available to All Solvers

pub const START_BIT: ThreadPaint = 0b0100_0000_0000_0000;
pub const FINISH_BIT: ThreadPaint = 0b1000_0000_0000_0000;
pub const NUM_THREADS: usize = 4;
pub const NUM_DIRECTIONS: usize = 4;
pub const THREAD_TAG_OFFSET: usize = 4;
pub const NUM_GATHER_FINISHES: usize = 4;
pub const INITIAL_PATH_LEN: usize = 1024;
pub const THREAD_MASK: ThreadPaint = 0b1111_0000;
pub const ZERO_THREAD: ThreadPaint = 0b0001_0000;
pub const ONE_THREAD: ThreadPaint = 0b0010_0000;
pub const TWO_THREAD: ThreadPaint = 0b0100_0000;
pub const THREE_THREAD: ThreadPaint = 0b1000_0000;
pub const ERROR_THREAD: ThreadPaint = 0b0000_0000;
pub const THREAD_MASKS: [ThreadPaint; 4] = [ZERO_THREAD, ONE_THREAD, TWO_THREAD, THREE_THREAD];

pub const CLEAR_CACHE: ThreadCache = 0b0001_1111_1111_0000;
pub const CACHE_MASK: ThreadCache = 0b1111_0000_0000;
pub const ZERO_SEEN: ThreadCache = 0b0001_0000_0000;
pub const ONE_SEEN: ThreadCache = 0b0010_0000_0000;
pub const TWO_SEEN: ThreadCache = 0b0100_0000_0000;
pub const THREE_SEEN: ThreadCache = 0b1000_0000_0000;

pub const ANSI_RED: &str = "\x1b[38;5;1m█\x1b[0m";
pub const ANSI_GRN: &str = "\x1b[38;5;2m█\x1b[0m";
pub const ANSI_YEL: &str = "\x1b[38;5;3m█\x1b[0m";
pub const ANSI_BLU: &str = "\x1b[38;5;4m█\x1b[0m";
pub const ANSI_PRP: &str = "\x1b[38;5;183m█\x1b[0m";
pub const ANSI_MAG: &str = "\x1b[38;5;201m█\x1b[0m";
pub const ANSI_CYN: &str = "\x1b[38;5;87m█\x1b[0m";
pub const ANSI_WIT: &str = "\x1b[38;5;231m█\x1b[0m";
pub const ANSI_PRP_RED: &str = "\x1b[38;5;204m█\x1b[0m";
pub const ANSI_BLU_MAG: &str = "\x1b[38;5;105m█\x1b[0m";
pub const ANSI_RED_GRN_BLU: &str = "\x1b[38;5;121m█\x1b[0m";
pub const ANSI_GRN_PRP: &str = "\x1b[38;5;106m█\x1b[0m";
pub const ANSI_GRN_BLU_PRP: &str = "\x1b[38;5;60m█\x1b[0m";
pub const ANSI_RED_GRN_PRP: &str = "\x1b[38;5;105m█\x1b[0m";
pub const ANSI_RED_BLU_PRP: &str = "\x1b[38;5;89m█\x1b[0m";
pub const ANSI_DRK_BLU_MAG: &str = "\x1b[38;5;57m█\x1b[0m";
pub const ANSI_BOLD: &str = "\x1b[1m";
pub const ANSI_NIL: &str = "\x1b[0m";
pub const ANSI_NO_SOLUTION: &str = "\x1b[38;5;15m\x1b[48;255;0;0m╳ no thread won..\x1b[0m";
pub const ANSI_FINISH: &str = "\x1b[1m\x1b[38;5;87mF\x1b[0m";
pub const ANSI_START: &str = "\x1b[1m\x1b[38;5;87mS\x1b[0m";
pub const ALL_THREADS_FAILED_INDEX: usize = 0;
// Threads Overlaps. The zero thread is the zero index bit with a value of 1.
pub static THREAD_COLORS: [&str; 16] = [
    ANSI_NO_SOLUTION, // 0b0000
    ANSI_RED,         // 0b0001
    ANSI_GRN,         // 0b0010
    ANSI_YEL,         // 0b0011
    ANSI_BLU,         // 0b0100
    ANSI_MAG,         // 0b0101
    ANSI_CYN,         // 0b0110
    ANSI_RED_GRN_BLU, // 0b0111
    ANSI_PRP,         // 0b1000
    ANSI_PRP_RED,     // 0b1001
    ANSI_GRN_PRP,     // 0b1010
    ANSI_RED_GRN_PRP, // 0b1011
    ANSI_DRK_BLU_MAG, // 0b1100
    ANSI_RED_BLU_PRP, // 0b1101
    ANSI_GRN_BLU_PRP, // 0b1110
    ANSI_WIT,         // 0b1111
];
pub const OVERLAP_KEY_AND_MESSAGE_HEIGHT: i32 = 9;
pub const SOLVER_SPEEDS: [SolveSpeedUnit; 8] = [0, 20000, 10000, 5000, 2000, 1000, 500, 250];
