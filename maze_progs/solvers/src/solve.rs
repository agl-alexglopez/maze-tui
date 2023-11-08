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
use ratatui::{
    buffer::Cell,
    style::{Color as RatColor, Modifier},
};
use std::io::{self};

// Types available to all solvers.
pub type ThreadPaint = u32;
pub type ThreadCache = u32;
pub type SolveSpeedUnit = u64;
pub struct ThreadGuide {
    pub index: usize,
    pub paint: ThreadPaint,
    pub cache: ThreadCache,
    pub start: maze::Point,
    pub speed: SolveSpeedUnit,
}

// Read Only Data Available to All Solvers
pub const START_BIT: ThreadPaint = 0x40000000;
pub const FINISH_BIT: ThreadPaint = 0x80000000;
pub const NUM_THREADS: usize = 4;
pub const NUM_DIRECTIONS: usize = 4;
pub const THREAD_TAG_OFFSET: usize = 4;
pub const NUM_GATHER_FINISHES: usize = 4;
pub const INITIAL_PATH_LEN: usize = 1024;
pub const THREAD_MASK: ThreadPaint = 0xFFFFFF;
pub const RED_MASK: ThreadPaint = 0xFF0000;
pub const RED_SHIFT: ThreadPaint = 16;
pub const GREEN_MASK: ThreadPaint = 0xFF00;
pub const GREEN_SHIFT: ThreadPaint = 8;
pub const BLUE_MASK: ThreadPaint = 0xFF;
// Credit to Caesar on StackOverflow for writing the program to find this tetrad of colors.
pub const THREAD_MASKS: [ThreadPaint; 4] = [0x880044, 0x766002, 0x009531, 0x010a88];
pub const CACHE_MASK: ThreadCache = 0xF000000;
pub const ZERO_SEEN: ThreadCache = 0x1000000;
pub const ONE_SEEN: ThreadCache = 0x2000000;
pub const TWO_SEEN: ThreadCache = 0x4000000;
pub const THREE_SEEN: ThreadCache = 0x8000000;
pub const THREAD_CACHES: [ThreadCache; 4] = [ZERO_SEEN, ONE_SEEN, TWO_SEEN, THREE_SEEN];
pub const SOLVER_SPEEDS: [SolveSpeedUnit; 8] = [0, 20000, 10000, 5000, 2000, 1000, 500, 250];

///
/// Logical helpers for bitwise operations.
///

#[inline]
pub fn is_start(square: maze::Square) -> bool {
    (square & START_BIT) != 0
}

#[inline]
pub fn is_finish(square: maze::Square) -> bool {
    (square & FINISH_BIT) != 0
}

#[inline]
pub fn is_color(square: maze::Square) -> bool {
    (square & THREAD_MASK) != 0
}

#[inline]
pub fn is_first(square: maze::Square) -> bool {
    (square & CACHE_MASK) == 0
}

#[inline]
fn thread_rgb(square: maze::Square) -> RatColor {
    RatColor::Rgb(
        ((square & RED_MASK) >> RED_SHIFT) as u8,
        ((square & GREEN_MASK) >> GREEN_SHIFT) as u8,
        (square & BLUE_MASK) as u8,
    )
}

#[inline]
fn is_start_or_finish(square: maze::Square) -> bool {
    (square & (START_BIT | FINISH_BIT)) != 0
}

#[inline]
fn is_valid_start_or_finish(maze: &maze::Maze, choice: maze::Point) -> bool {
    choice.row > 0
        && choice.row < maze.rows() - 1
        && choice.col > 0
        && choice.col < maze.cols() - 1
        && maze.path_at(choice.row, choice.col)
        && !is_finish(maze.get(choice.row, choice.col))
        && !is_start(maze.get(choice.row, choice.col))
}

#[inline]
fn color_index(square: maze::Square) -> usize {
    ((square & THREAD_MASK) >> THREAD_TAG_OFFSET) as usize
}

///
/// Setup functions for starting and finishing a solver section.
///

pub fn reset_solve(maze: &mut maze::Maze) {
    for square in maze.as_slice_mut().iter_mut() {
        if (*square & maze::PATH_BIT) != 0 {
            *square = maze::PATH_BIT;
        }
    }
}

pub fn set_corner_starts(maze: &maze::Maze) -> [maze::Point; 4] {
    let mut point1: maze::Point = maze::Point { row: 1, col: 1 };
    if maze.wall_at(point1.row, point1.col) {
        point1 = find_nearest_square(maze, point1);
    }
    let mut point2: maze::Point = maze::Point {
        row: 1,
        col: maze.cols() - 2,
    };
    if maze.wall_at(point2.row, point2.col) {
        point2 = find_nearest_square(maze, point2);
    }
    let mut point3: maze::Point = maze::Point {
        row: maze.rows() - 2,
        col: 1,
    };
    if maze.wall_at(point3.row, point3.col) {
        point3 = find_nearest_square(maze, point3);
    }
    let mut point4: maze::Point = maze::Point {
        row: maze.rows() - 2,
        col: maze.cols() - 2,
    };
    if maze.wall_at(point4.row, point4.col) {
        point4 = find_nearest_square(maze, point4);
    }
    [point1, point2, point3, point4]
}

pub fn pick_random_point(maze: &maze::Maze) -> maze::Point {
    let mut gen = thread_rng();
    let choice = maze::Point {
        row: gen.gen_range(1..maze.rows() - 2),
        col: gen.gen_range(1..maze.cols() - 2),
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
    for r in 1..maze.rows() - 1 {
        for c in 1..maze.cols() - 1 {
            let cur = maze::Point { row: r, col: c };
            if is_valid_start_or_finish(maze, cur) {
                return cur;
            }
        }
    }
    print_paths(maze);
    print::maze_panic!("Could not place a point in this maze. Was it built correctly?");
}

///
/// Playback and animation based logic for interacting with TUI buffer.
///

pub fn decode_square(wall_row: &[char], square: maze::Square) -> Cell {
    // We have some special printing for the finish square. Not here.
    if is_finish(square) {
        Cell {
            symbol: 'F'.to_string(),
            fg: RatColor::Indexed(key::ANSI_CYN),
            bg: thread_rgb(square),
            underline_color: RatColor::Reset,
            modifier: Modifier::BOLD | Modifier::SLOW_BLINK,
            skip: false,
        }
    } else if is_start(square) {
        Cell {
            symbol: 'S'.to_string(),
            fg: RatColor::Indexed(key::ANSI_CYN),
            bg: RatColor::Reset,
            underline_color: RatColor::Reset,
            modifier: Modifier::BOLD,
            skip: false,
        }
    } else if is_color(square) {
        Cell {
            symbol: "█".to_string(),
            fg: thread_rgb(square),
            bg: RatColor::Reset,
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        }
    } else if maze::is_wall(square) {
        Cell {
            symbol: wall_row[((square & maze::WALL_MASK) >> maze::WALL_SHIFT) as usize].to_string(),
            fg: RatColor::Reset,
            bg: RatColor::Reset,
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        }
    } else if maze::is_path(square) {
        Cell {
            symbol: ' '.to_string(),
            fg: RatColor::Reset,
            bg: RatColor::Reset,
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        }
    } else {
        maze_panic!("Uncategorized maze square! Check the bits.");
    }
}

// This is really bad, there must be a better way. Coloring halves correctly is a challenge.
pub fn decode_mini_path(maze: &maze::Blueprint, p: maze::Point) -> Cell {
    let square = maze.get(p.row, p.col);
    let this_color = thread_rgb(square);
    if is_start_or_finish(square) {
        return Cell {
            symbol: '▀'.to_string(),
            fg: RatColor::Indexed(key::ANSI_CYN),
            bg: this_color,
            underline_color: RatColor::Reset,
            modifier: Modifier::SLOW_BLINK,
            skip: false,
        };
    }
    // An odd square will always have something above but we could be a path or wall.
    if p.row % 2 != 0 {
        if maze.path_at(p.row, p.col) {
            if maze.path_at(p.row - 1, p.col) {
                let neighbor_square = maze.get(p.row - 1, p.col);
                if is_start_or_finish(neighbor_square) {
                    // A special square is our neighbor.
                    return Cell {
                        symbol: '▀'.to_string(),
                        fg: RatColor::Indexed(key::ANSI_CYN),
                        bg: this_color,
                        underline_color: RatColor::Reset,
                        modifier: Modifier::SLOW_BLINK,
                        skip: false,
                    };
                }
                // Another thread may be above us so grab the color invariantly just in case.
                return Cell {
                    symbol: '▀'.to_string(),
                    fg: thread_rgb(neighbor_square),
                    bg: this_color,
                    underline_color: RatColor::Reset,
                    modifier: Modifier::empty(),
                    skip: false,
                };
            }
            // A wall is above a path so no extra color logic needed.
            return Cell {
                symbol: '▀'.to_string(),
                fg: RatColor::Reset,
                bg: this_color,
                underline_color: RatColor::Reset,
                modifier: Modifier::empty(),
                skip: false,
            };
        }
        // The only odd wall sqares are those connecting two even rows above and below.
        return Cell {
            symbol: '█'.to_string(),
            fg: RatColor::Reset,
            bg: RatColor::Reset,
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        };
    }
    // This is an even row. Let's run the logic for both paths and walls being here.
    if maze.path_at(p.row, p.col) {
        if maze.path_at(p.row + 1, p.col) {
            let neighbor_square = maze.get(p.row + 1, p.col);
            if is_start_or_finish(neighbor_square) {
                // A special neighbor is below us so we must split the square colors.
                return Cell {
                    symbol: '▀'.to_string(),
                    fg: RatColor::Indexed(key::ANSI_CYN),
                    bg: this_color,
                    underline_color: RatColor::Reset,
                    modifier: Modifier::SLOW_BLINK,
                    skip: false,
                };
            }
            // Another thread may be below us so grab the color invariantly just in case.
            return Cell {
                symbol: '▀'.to_string(),
                fg: thread_rgb(neighbor_square),
                bg: this_color,
                underline_color: RatColor::Reset,
                modifier: Modifier::empty(),
                skip: false,
            };
        }
        // A wall is below a path so not coloring of the block this time.
        return Cell {
            symbol: '▄'.to_string(),
            fg: RatColor::Reset,
            bg: this_color,
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        };
    }
    // This is a wall square in an even row. A path or other wall can be below.
    if p.row + 1 < maze.rows && maze.path_at(p.row + 1, p.col) {
        let neighbor_square = maze.get(p.row + 1, p.col);
        if is_start_or_finish(neighbor_square) {
            // The wall has a special neighbor so color halves appropriately.
            return Cell {
                symbol: '▀'.to_string(),
                fg: RatColor::Reset,
                bg: RatColor::Indexed(key::ANSI_CYN),
                underline_color: RatColor::Reset,
                modifier: Modifier::SLOW_BLINK,
                skip: false,
            };
        }
        // The wall may have a thread below so grab the color just in case.
        return Cell {
            symbol: '▀'.to_string(),
            fg: RatColor::Reset,
            bg: thread_rgb(neighbor_square),
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        };
    }
    // Edge case. If a wall is below us in an even row it will print the full block for us when we
    // get to it. If not we are at the end of the maze and this is the right square to print.
    Cell {
        symbol: '▀'.to_string(),
        fg: RatColor::Reset,
        bg: RatColor::Reset,
        underline_color: RatColor::Reset,
        modifier: Modifier::empty(),
        skip: false,
    }
}

///
/// Cursor based display logic for printing and coloring.
///

pub fn print_paths(maze: &maze::Maze) {
    if maze.style_index() == (maze::MazeStyle::Mini as usize) {
        for r in 0..maze.rows() {
            for c in 0..maze.cols() {
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
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
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
    let square = maze.get(point.row, point.col);
    // We have some special printing for the finish square. Not here.
    if is_finish(square) {
        let ansi = key::thread_color_code(color_index(square));
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
    if is_start(square) {
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
    if is_color(square) {
        let thread_color: key::ThreadColor = key::thread_color(color_index(square));
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
    if maze::is_wall(square) {
        match execute!(io::stdout(), Print(maze.wall_char(square)),) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not print wall."),
        }
    }
    if maze::is_path(square) {
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
    let square = maze.get(point.row, point.col);
    if is_finish(square) {
        let ansi = key::thread_color_code(color_index(square));
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
    if is_start(square) {
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
    if is_color(square) {
        let thread_color: key::ThreadColor = key::thread_color(color_index(square));
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
    if maze::is_wall(square) {
        match queue!(io::stdout(), Print(maze.wall_char(square)),) {
            Ok(_) => return,
            Err(_) => maze_panic!("Could not print wall."),
        }
    }
    if maze::is_path(square) {
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
    let square = maze.get(point.row, point.col);
    let this_color = key::thread_color_code(color_index(square));
    if is_start_or_finish(square) {
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
        if maze.path_at(point.row - 1, point.col) {
            let neighbor_square = maze.get(point.row - 1, point.col);
            if is_start_or_finish(neighbor_square) {
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
                SetForegroundColor(Color::AnsiValue(key::thread_color_code(color_index(
                    neighbor_square
                )))),
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
    if maze.path_at(point.row + 1, point.col) {
        let neighbor_square = maze.get(point.row + 1, point.col);
        if is_start_or_finish(neighbor_square) {
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
            SetForegroundColor(Color::AnsiValue(key::thread_color_code(color_index(
                neighbor_square
            )))),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // A wall is below us
    execute!(
        io::stdout(),
        SetBackgroundColor(Color::AnsiValue(this_color)),
        Print('▄'),
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
    let square = maze.get(point.row, point.col);
    let this_color = key::thread_color_code(color_index(square));
    if is_start_or_finish(square) {
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
    if maze::is_wall(square) {
        queue!(io::stdout(), Print(maze.wall_char(square)),).expect("printer broke.");
        return;
    }
    if point.row % 2 != 0 {
        if maze.path_at(point.row - 1, point.col) {
            let neighbor_square = maze.get(point.row - 1, point.col);
            if is_start_or_finish(neighbor_square) {
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
                SetForegroundColor(Color::AnsiValue(key::thread_color_code(color_index(
                    neighbor_square
                )))),
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
    if maze.path_at(point.row + 1, point.col) {
        let neighbor_square = maze.get(point.row + 1, point.col);
        if is_start_or_finish(neighbor_square) {
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
            SetForegroundColor(Color::AnsiValue(key::thread_color_code(color_index(
                neighbor_square
            )))),
            SetBackgroundColor(Color::AnsiValue(this_color)),
            Print('▀'),
            ResetColor
        )
        .expect("printer broke.");
        return;
    }
    // A wall is below
    queue!(
        io::stdout(),
        SetBackgroundColor(Color::AnsiValue(this_color)),
        Print('▄'),
        ResetColor
    )
    .expect("printer broke.");
}
