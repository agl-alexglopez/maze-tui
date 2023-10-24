use crossterm::{
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
};
use key;
use maze;
use print::maze_panic;
use rand::prelude::*;
use std::io::{self};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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

pub struct MaxMap {
    pub max: u64,
    pub distances: HashMap<maze::Point, u64>,
}

impl MaxMap {
    pub fn new(p: maze::Point, m: u64) -> Self {
        Self {
            max: m,
            distances: HashMap::from([(p, m)]),
        }
    }
    pub fn default() -> Self {
        Self {
            max: 0,
            distances: HashMap::default(),
        }
    }
}

pub struct Solver {
    pub maze: maze::Maze,
    pub win: Option<usize>,
    pub win_path: Vec<(maze::Point, ThreadPaint)>,
    pub map: MaxMap,
    pub count: usize,
}

impl Solver {
    pub fn new(boxed_maze: maze::Maze) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            maze: boxed_maze,
            win: None,
            win_path: Vec::default(),
            map: MaxMap::default(),
            count: 0,
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
    if maze.style_index() == (maze::MazeStyle::Mini as usize) {
        for r in 0..maze.row_size() {
            for c in 0..maze.col_size() {
                print_mini_point(maze, maze::Point { row: r, col: c });
            }
            match queue!(io::stdout(), Print('\n'),) {
                Ok(_) => {}
                Err(_) => maze_panic!("Could not print newline."),
            }
        }
        print::flush();
        return;
    }
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            print_point(maze, maze::Point { row: r, col: c });
        }
        match queue!(io::stdout(), Print('\n'),) {
            Ok(_) => {}
            Err(_) => maze_panic!("Could not print newline."),
        }
    }
    print::flush();
}

pub fn flush_cursor_path_coordinate(maze: &maze::Maze, point: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: point.row,
            col: point.col,
        },
        maze.offset(),
    );
    let square = maze[point.row as usize][point.col as usize];
    // We have some special printing for the finish square. Not here.
    if (square & FINISH_BIT) != 0 {
        let ansi = key::thread_color_code(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
        match execute!(
            io::stdout(),
            SetAttribute(Attribute::SlowBlink),
            SetAttribute(Attribute::Bold),
            SetBackgroundColor(Color::AnsiValue(ansi)),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('F'),
            ResetColor
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not mark Finish square"),
        }
    }
    if (square & START_BIT) != 0 {
        match execute!(
            io::stdout(),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('S'),
            ResetColor
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not mark Start square."),
        }
    }
    if (square & THREAD_MASK) != 0 {
        let thread_color: key::ThreadColor =
            key::thread_color(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
        match execute!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(thread_color.ansi)),
            Print(thread_color.block),
            ResetColor,
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not mark thread color."),
        }
    }
    if (square & maze::PATH_BIT) == 0 {
        match execute!(
            io::stdout(),
            Print(maze.wall_char((square & maze::WALL_MASK) as usize)),
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not print wall."),
        }
    }
    if (square & maze::PATH_BIT) != 0 {
        match execute!(io::stdout(), Print(' '),) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not print path."),
        }
    }
    maze_panic!("Uncategorized maze square! Check the bits.");
}

pub fn print_point(maze: &maze::Maze, point: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: point.row,
            col: point.col,
        },
        maze.offset(),
    );
    let square = &maze[point.row as usize][point.col as usize];
    if (square & FINISH_BIT) != 0 {
        let ansi = key::thread_color_code(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
        match queue!(
            io::stdout(),
            SetAttribute(Attribute::SlowBlink),
            SetAttribute(Attribute::Bold),
            SetBackgroundColor(Color::AnsiValue(ansi)),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('F'),
            ResetColor
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not mark Finish square"),
        }
    }
    if (square & START_BIT) != 0 {
        match queue!(
            io::stdout(),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('S'),
            ResetColor
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not mark Start square."),
        }
    }
    if (square & THREAD_MASK) != 0 {
        let thread_color: key::ThreadColor =
            key::thread_color(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
        match queue!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(thread_color.ansi)),
            Print(thread_color.block),
            ResetColor,
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not mark thread color."),
        }
    }
    if (square & maze::PATH_BIT) == 0 {
        match queue!(
            io::stdout(),
            Print(maze.wall_char((square & maze::WALL_MASK) as usize)),
        ) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not print wall."),
        }
    }
    if (square & maze::PATH_BIT) != 0 {
        match queue!(io::stdout(), Print(' '),) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not print path."),
        }
    }
    maze_panic!("Uncategorized maze square! Check the bits.");
}

// These printers for the Mini wall style are brutal. If you ever think of something better, fix.

pub fn flush_mini_path_coordinate(maze: &maze::Maze, point: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: point.row / 2,
            col: point.col,
        },
        maze.offset(),
    );
    let square = maze[point.row as usize][point.col as usize];
    let this_color = key::thread_color_code(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
    if (square & (FINISH_BIT | START_BIT)) != 0 {
        execute!(
            io::stdout(),
            SetAttribute(Attribute::SlowBlink),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // This is a path square. We should never be asked to print a wall from a solver animation?
    if point.row % 2 != 0 {
        if (maze[(point.row - 1) as usize][point.col as usize] & maze::PATH_BIT) != 0 {
            let neighbor_square = maze[(point.row - 1) as usize][point.col as usize];
            if (neighbor_square & (START_BIT | FINISH_BIT)) != 0 {
                execute!(
                    io::stdout(),
                    SetAttribute(Attribute::SlowBlink),
                    SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
                    SetBackgroundColor(Color::AnsiValue(this_color)),
                    Print('▀'),
                    ResetColor
                )
                .expect("printer broke.");
                return;
            }
            execute!(
                io::stdout(),
                SetForegroundColor(Color::AnsiValue(key::thread_color_code(
                    ((neighbor_square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize,
                ))),
                SetBackgroundColor(Color::AnsiValue(this_color)),
                Print('▀'),
                ResetColor
            )
            .expect("printer broke.");
            return;
        }
        // A wall is above us.
        execute!(
            io::stdout(),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // Even rows but this still must be a path.
    if (maze[(point.row + 1) as usize][point.col as usize] & maze::PATH_BIT) != 0 {
        let neighbor_square = maze[(point.row + 1) as usize][point.col as usize];
        if (neighbor_square & (START_BIT | FINISH_BIT)) != 0 {
            execute!(
                io::stdout(),
                SetAttribute(Attribute::SlowBlink),
                SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
                SetBackgroundColor(Color::AnsiValue(this_color)),
                Print('▀'),
                ResetColor
            )
            .expect("printer broke.");
            return;
        }
        execute!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(key::thread_color_code(
                ((neighbor_square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize
            ))),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // A wall is above us
    execute!(
        io::stdout(),
        SetBackgroundColor(Color::AnsiValue(this_color)),
        Print('▀'),
        ResetColor
    )
    .expect("printer broke.");
}

pub fn print_mini_point(maze: &maze::Maze, point: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: point.row / 2,
            col: point.col,
        },
        maze.offset(),
    );
    let square = maze[point.row as usize][point.col as usize];
    let this_color = key::thread_color_code(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
    if (square & (FINISH_BIT | START_BIT)) != 0 {
        queue!(
            io::stdout(),
            SetAttribute(Attribute::SlowBlink),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    if square & maze::PATH_BIT == 0 {
        queue!(
            io::stdout(),
            Print(maze.wall_char((square & maze::WALL_MASK) as usize)),
        )
        .expect("printer broke.");
        return;
    }
    if point.row % 2 != 0 {
        if (maze[(point.row - 1) as usize][point.col as usize] & maze::PATH_BIT) != 0 {
            let neighbor_square = maze[(point.row - 1) as usize][point.col as usize];
            if (neighbor_square & (START_BIT | FINISH_BIT)) != 0 {
                queue!(
                    io::stdout(),
                    SetAttribute(Attribute::SlowBlink),
                    SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
                    SetBackgroundColor(Color::AnsiValue(this_color)),
                    Print('▀'),
                    ResetColor
                )
                .expect("printer broke.");
                return;
            }
            queue!(
                io::stdout(),
                SetForegroundColor(Color::AnsiValue(key::thread_color_code(
                    ((neighbor_square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize,
                ))),
                SetBackgroundColor(Color::AnsiValue(this_color)),
                Print('▀'),
                ResetColor
            )
            .expect("printer broke.");
            return;
        }
        // A wall is above us.
        queue!(
            io::stdout(),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // Even rows but this still must be a path.
    if (maze[(point.row + 1) as usize][point.col as usize] & maze::PATH_BIT) != 0 {
        let neighbor_square = maze[(point.row + 1) as usize][point.col as usize];
        if (neighbor_square & (START_BIT | FINISH_BIT)) != 0 {
            queue!(
                io::stdout(),
                SetAttribute(Attribute::SlowBlink),
                SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
                SetBackgroundColor(Color::AnsiValue(this_color)),
                Print('▀'),
                ResetColor
            )
            .expect("printer broke.");
            return;
        }
        queue!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(key::thread_color_code(
                ((neighbor_square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize
            ))),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // A wall is above us.
    queue!(
        io::stdout(),
        SetBackgroundColor(Color::AnsiValue(this_color)),
        Print('▀'),
        ResetColor
    )
    .expect("printer broke.");
}

// Because we are using half blocks we need a different function to invert colors for dark mode.
pub fn flush_dark_mini_path_coordinate(maze: &maze::Maze, point: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: point.row / 2,
            col: point.col,
        },
        maze.offset(),
    );
    let square = maze[point.row as usize][point.col as usize];
    let this_color = key::thread_color_code(((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize);
    if (square & (FINISH_BIT | START_BIT)) != 0 {
        execute!(
            io::stdout(),
            SetAttribute(Attribute::SlowBlink),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // This is a path square. We should never be asked to print a wall from a solver animation?
    if point.row % 2 != 0 {
        if (maze[(point.row - 1) as usize][point.col as usize] & maze::PATH_BIT) != 0 {
            let neighbor_square = maze[(point.row - 1) as usize][point.col as usize];
            if (neighbor_square & (START_BIT | FINISH_BIT)) != 0 {
                execute!(
                    io::stdout(),
                    SetAttribute(Attribute::SlowBlink),
                    SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
                    SetBackgroundColor(Color::AnsiValue(this_color)),
                    Print('▀'),
                    ResetColor
                )
                .expect("printer broke.");
                return;
            }
            execute!(
                io::stdout(),
                SetForegroundColor(Color::AnsiValue(key::thread_color_code(
                    ((neighbor_square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize,
                ))),
                SetBackgroundColor(Color::AnsiValue(this_color)),
                Print('▀'),
                ResetColor
            )
            .expect("printer broke.");
            return;
        }
        // A wall above but we are in dark mode so make it black. Path is now square.
        execute!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(this_color)),
            Print('▄'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // Even rows but this still must be a path.
    if (maze[(point.row + 1) as usize][point.col as usize] & maze::PATH_BIT) != 0 {
        let neighbor_square = maze[(point.row + 1) as usize][point.col as usize];
        if (neighbor_square & (START_BIT | FINISH_BIT)) != 0 {
            execute!(
                io::stdout(),
                SetAttribute(Attribute::SlowBlink),
                SetForegroundColor(Color::AnsiValue(key::ANSI_CYN)),
                SetBackgroundColor(Color::AnsiValue(this_color)),
                Print('▀'),
                ResetColor
            )
            .expect("printer broke.");
            return;
        }
        execute!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(key::thread_color_code(
                ((neighbor_square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize
            ))),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // A wall above but we are in dark mode so make it black. Path is now square.
    execute!(
        io::stdout(),
        SetForegroundColor(Color::AnsiValue(this_color)),
        Print('▀'),
        ResetColor
    )
    .expect("printer broke.");
}

pub fn deluminate_maze(maze: &maze::Maze) {
    if maze.style_index() == (maze::MazeStyle::Mini as usize) {
        for r in 0..(maze.row_size() + 1) / 2 {
            for c in 0..maze.col_size() {
                let p = maze::Point { row: r, col: c };
                print::set_cursor_position(p, maze.offset());
                match queue!(io::stdout(), Print(' '),) {
                    Ok(_) => {}
                    Err(_) => maze_panic!("Could not print path."),
                }
            }
        }
    } else {
        for r in 0..maze.row_size() {
            for c in 0..maze.col_size() {
                let p = maze::Point { row: r, col: c };
                print::set_cursor_position(p, maze.offset());
                match queue!(io::stdout(), Print(' '),) {
                    Ok(_) => {}
                    Err(_) => maze_panic!("Could not print path."),
                }
            }
        }
    }
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

pub const SOLVER_SPEEDS: [SolveSpeedUnit; 8] = [0, 20000, 10000, 5000, 2000, 1000, 500, 250];
