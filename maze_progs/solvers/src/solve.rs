use crossterm::{
    queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
};
use print::maze_panic;
use rand::prelude::*;
use std::io::{self};
use std::sync::{Arc, Mutex};
// Types available to all solvers.

pub type ThreadPaint = u16;
pub type ThreadCache = u16;
pub type SolveSpeedUnit = u64;

pub struct ThreadGuide {
    pub index: usize,
    pub paint: ThreadPaint,
    pub start: maze::Point,
    pub speed: SolveSpeedUnit,
}

pub struct ThreadColor {
    pub block: &'static str,
    pub code: u8,
}

pub struct Solver {
    pub maze: maze::BoxMaze,
    pub win: Option<usize>,
}

impl Solver {
    pub fn new(boxed_maze: maze::BoxMaze) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            maze: boxed_maze,
            win: None,
        }))
    }
}

pub type SolverMonitor = Arc<Mutex<Solver>>;

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
    [point1, point2, point3, point4]
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
    print::maze_panic!("Could not place a point in this maze. Was it built correctly?");
}

pub fn print_paths(maze: &maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            print_point(maze, maze::Point { row: r, col: c });
        }
        println!();
    }
    print::flush();
}

pub fn flush_cursor_path_coordinate(maze: &maze::Maze, point: maze::Point) {
    print::set_cursor_position(point);
    print_point(maze, point);
    print::flush();
}

pub fn print_point(maze: &maze::Maze, point: maze::Point) {
    let square = &maze[point.row as usize][point.col as usize];
    // We have some special printing for the finish square. Not here.
    if (square & FINISH_BIT) != 0 {
        let av = THREAD_COLORS[((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize].code;
        match queue!(
            io::stdout(),
            SetAttribute(Attribute::SlowBlink),
            SetAttribute(Attribute::Bold),
            SetBackgroundColor(Color::AnsiValue(av)),
            SetForegroundColor(Color::AnsiValue(ANSI_CYN)),
            Print("F".to_string()),
            ResetColor
        ) {
            Ok(_) => {}
            Err(_) => maze_panic!("Could not mark Finish square"),
        }
        return;
    }
    if (square & START_BIT) != 0 {
        print!("{}", ANSI_START);
        return;
    }
    if (square & THREAD_MASK) != 0 {
        let thread_color: ThreadPaint = (square & THREAD_MASK) >> THREAD_TAG_OFFSET;
        print!("{}", THREAD_COLORS[thread_color as usize].block);
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
    print::maze_panic!("Uncategorized maze square! Check the bits.");
}

pub fn deluminate_maze(rows: i32, cols: i32) {
    for r in 0..rows {
        for c in 0..cols {
            let p = maze::Point { row: r, col: c };
            print::set_cursor_position(p);
            print!(" ");
        }
    }
}

pub fn print_hunt_solution_message(winning_index: Option<usize>) {
    match winning_index {
        Some(i) => print!(
            "{} THREAD {} WINS!",
            THREAD_COLORS[(THREAD_MASKS[i] >> THREAD_TAG_OFFSET) as usize].block,
            i
        ),
        None => print!("{}", THREAD_COLORS[ALL_THREADS_FAILED_INDEX].block),
    }
}

pub fn print_gather_solution_message() {
    for mask in &THREAD_MASKS {
        print!(
            "{}",
            THREAD_COLORS[(mask >> THREAD_TAG_OFFSET) as usize].block
        );
    }
    println!(" All threads found their finish squares!");
}

pub fn print_overlap_key(mut pos: maze::Point) {
    print::set_cursor_position(pos);
    pos.row += 1;
    println!("┌────────────────────────────────────────────────────────────────┐");
    print::set_cursor_position(pos);
    pos.row += 1;
    println!("│     Overlap Key: 3_THREAD | 2_THREAD | 1_THREAD | 0_THREAD     │");
    print::set_cursor_position(pos);
    pos.row += 1;
    println!("├────────────┬────────────┬────────────┬────────────┬────────────┤");
    print::set_cursor_position(pos);
    pos.row += 1;
    println!(
        "│ {} = 0      │ {} = 1      │ {} = 1|0    │ {} = 2      │ {} = 2|0    │",
        THREAD_COLORS[1].block,
        THREAD_COLORS[2].block,
        THREAD_COLORS[3].block,
        THREAD_COLORS[4].block,
        THREAD_COLORS[5].block
    );
    print::set_cursor_position(pos);
    pos.row += 1;
    println!("├────────────┼────────────┼────────────┼────────────┼────────────┤");
    print::set_cursor_position(pos);
    pos.row += 1;
    println!(
        "│ {} = 2|1    │ {} = 2|1|0  │ {} = 3      │ {} = 3|0    │ {} = 3|1    │",
        THREAD_COLORS[6].block,
        THREAD_COLORS[7].block,
        THREAD_COLORS[8].block,
        THREAD_COLORS[9].block,
        THREAD_COLORS[10].block
    );
    print::set_cursor_position(pos);
    pos.row += 1;
    println!("├────────────┼────────────┼────────────┼────────────┼────────────┤");
    print::set_cursor_position(pos);
    pos.row += 1;
    println!(
        "│ {} = 3|1|0  │ {} = 3|2    │ {} = 3|2|0  │ {} = 3|2|1  │ {} = 3|2|1|0│",
        THREAD_COLORS[11].block,
        THREAD_COLORS[12].block,
        THREAD_COLORS[13].block,
        THREAD_COLORS[14].block,
        THREAD_COLORS[15].block
    );
    print::set_cursor_position(pos);
    println!("└────────────┴────────────┴────────────┴────────────┴────────────┘");
}

// Private Module Function

fn is_valid_start_or_finish(maze: &maze::Maze, choice: maze::Point) -> bool {
    choice.row > 0
        && choice.row < maze.row_size() - 1
        && choice.col > 0
        && choice.col < maze.col_size() - 1
        && (maze[choice.row as usize][choice.col as usize] & maze::PATH_BIT) != 0
        && (maze[choice.row as usize][choice.col as usize] & FINISH_BIT) == 0
        && (maze[choice.row as usize][choice.col as usize] & START_BIT) == 0
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
pub const THREAD_MASKS: [ThreadPaint; 4] = [ZERO_THREAD, ONE_THREAD, TWO_THREAD, THREE_THREAD];

pub const CACHE_MASK: ThreadCache = 0b1111_0000_0000;

// The first four colors are the thread primitives that mix to form the rest.
pub const ANSI_RED: u8 = 1;
pub const ANSI_GRN: u8 = 2;
pub const ANSI_BLU: u8 = 4;
pub const ANSI_PRP: u8 = 183;
pub const ANSI_CYN: u8 = 14;
pub const ANSI_RED_BLOCK: &str = "\x1b[38;5;1m█\x1b[0m";
pub const ANSI_GRN_BLOCK: &str = "\x1b[38;5;2m█\x1b[0m";
pub const ANSI_YEL_BLOCK: &str = "\x1b[38;5;3m█\x1b[0m";
pub const ANSI_BLU_BLOCK: &str = "\x1b[38;5;4m█\x1b[0m";
pub const ANSI_PRP_BLOCK: &str = "\x1b[38;5;183m█\x1b[0m";
pub const ANSI_MAG_BLOCK: &str = "\x1b[38;5;201m█\x1b[0m";
pub const ANSI_CYN_BLOCK: &str = "\x1b[38;5;87m█\x1b[0m";
pub const ANSI_WIT_BLOCK: &str = "\x1b[38;5;231m█\x1b[0m";
pub const ANSI_PRP_RED_BLOCK: &str = "\x1b[38;5;204m█\x1b[0m";
pub const ANSI_RED_GRN_BLU_BLOCK: &str = "\x1b[38;5;121m█\x1b[0m";
pub const ANSI_GRN_PRP_BLOCK: &str = "\x1b[38;5;106m█\x1b[0m";
pub const ANSI_GRN_BLU_PRP_BLOCK: &str = "\x1b[38;5;60m█\x1b[0m";
pub const ANSI_RED_GRN_PRP_BLOCK: &str = "\x1b[38;5;105m█\x1b[0m";
pub const ANSI_RED_BLU_PRP_BLOCK: &str = "\x1b[38;5;89m█\x1b[0m";
pub const ANSI_DRK_BLU_MAG_BLOCK: &str = "\x1b[38;5;57m█\x1b[0m";
pub const ANSI_NO_SOLUTION: &str = "\x1b[38;5;15m\x1b[48;255;0;0m╳ no thread won..\x1b[0m";
pub const ANSI_START: &str = "\x1b[1m\x1b[38;5;87mS\x1b[0m";
pub const ALL_THREADS_FAILED_INDEX: usize = 0;
// Threads Overlaps. The zero thread is the zero index bit with a value of 1.
pub static THREAD_COLORS: [ThreadColor; 16] = [
    ThreadColor {
        // 0b0000
        block: ANSI_NO_SOLUTION,
        code: 0,
    },
    ThreadColor {
        // 0b0001
        block: ANSI_RED_BLOCK,
        code: ANSI_RED,
    },
    ThreadColor {
        // 0b0010
        block: ANSI_GRN_BLOCK,
        code: ANSI_GRN,
    },
    ThreadColor {
        // 0b0011
        block: ANSI_YEL_BLOCK,
        code: 3,
    },
    ThreadColor {
        // 0b0100
        block: ANSI_BLU_BLOCK,
        code: ANSI_BLU,
    },
    ThreadColor {
        // 0b0101
        block: ANSI_MAG_BLOCK,
        code: 201,
    },
    ThreadColor {
        // 0b0110
        block: ANSI_CYN_BLOCK,
        code: ANSI_CYN,
    },
    ThreadColor {
        // 0b0111
        block: ANSI_RED_GRN_BLU_BLOCK,
        code: 121,
    },
    ThreadColor {
        // 0b1000
        block: ANSI_PRP_BLOCK,
        code: ANSI_PRP,
    },
    ThreadColor {
        // 0b1001
        block: ANSI_PRP_RED_BLOCK,
        code: 204,
    },
    ThreadColor {
        // 0b1010
        block: ANSI_GRN_PRP_BLOCK,
        code: 106,
    },
    ThreadColor {
        // 0b1011
        block: ANSI_RED_GRN_PRP_BLOCK,
        code: 105,
    },
    ThreadColor {
        // 0b1100
        block: ANSI_DRK_BLU_MAG_BLOCK,
        code: 57,
    },
    ThreadColor {
        // 0b1101
        block: ANSI_RED_BLU_PRP_BLOCK,
        code: 89,
    },
    ThreadColor {
        // 0b1110
        block: ANSI_GRN_BLU_PRP_BLOCK,
        code: 60,
    },
    ThreadColor {
        // 0b1111
        block: ANSI_WIT_BLOCK,
        code: 231,
    },
];
pub const OVERLAP_KEY_AND_MESSAGE_HEIGHT: i32 = 9;
pub const SOLVER_SPEEDS: [SolveSpeedUnit; 8] = [0, 20000, 10000, 5000, 2000, 1000, 500, 250];
