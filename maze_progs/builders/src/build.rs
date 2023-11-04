use crossterm::{
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
};
use key;
use maze;
use print;
use print::maze_panic;
use ratatui::{
    buffer::Cell,
    style::{Color as RatColor, Modifier},
};
use std::io::{self};

use std::{thread, time};

pub type SpeedUnit = u64;
pub type BacktrackMarker = u16;

#[derive(PartialEq, Eq)]
pub enum ParityPoint {
    Even,
    Odd,
}

#[derive(Copy, Clone)]
pub struct BacktrackSymbol {
    pub arrow: char,
    pub ansi: u8,
}

// Any builders that choose to cache seen squares in place can use this bit.
pub const BUILDER_BIT: maze::Square = 0b0001_0000_0000_0000;
// Data that will help backtracker algorithms like recursive backtracker and Wilson's.
pub const MARKER_SHIFT: u8 = 4;
pub const NUM_DIRECTIONS: usize = 4;
pub const MARKERS_MASK: BacktrackMarker = 0b1111_0000;
pub const FROM_NORTH: BacktrackMarker = 0b0001_0000;
pub const FROM_EAST: BacktrackMarker = 0b0010_0000;
pub const FROM_SOUTH: BacktrackMarker = 0b0011_0000;
pub const FROM_WEST: BacktrackMarker = 0b0100_0000;
pub const ANSI_WHITE: u8 = 15;
pub static BACKTRACKING_SYMBOLS: [BacktrackSymbol; 5] = [
    BacktrackSymbol {
        // Origin
        arrow: ' ',
        ansi: 0,
    },
    BacktrackSymbol {
        // From North
        arrow: '↑',
        ansi: 1,
    },
    BacktrackSymbol {
        // From East
        arrow: '→',
        ansi: 2,
    },
    BacktrackSymbol {
        // From South
        arrow: '↓',
        ansi: 3,
    },
    BacktrackSymbol {
        // From West
        arrow: '←',
        ansi: 4,
    },
];

pub const BACKTRACKING_POINTS: [maze::Point; 5] = [
    maze::Point { row: 0, col: 0 },
    maze::Point { row: -2, col: 0 },
    maze::Point { row: 0, col: 2 },
    maze::Point { row: 2, col: 0 },
    maze::Point { row: 0, col: -2 },
];

pub const BACKTRACKING_HALF_POINTS: [maze::Point; 5] = [
    maze::Point { row: 0, col: 0 },
    maze::Point { row: -1, col: 0 },
    maze::Point { row: 0, col: 1 },
    maze::Point { row: 1, col: 0 },
    maze::Point { row: 0, col: -1 },
];

// Most builder algorithms will need to use these so leave them in one place.

// north, east, south, west
pub const GENERATE_DIRECTIONS: [maze::Point; 4] = [
    maze::Point { row: -2, col: 0 },
    maze::Point { row: 0, col: 2 },
    maze::Point { row: 2, col: 0 },
    maze::Point { row: 0, col: -2 },
];

// Control the speed steps of animation in microseconds here.
pub const BUILDER_SPEEDS: [SpeedUnit; 8] = [0, 5000, 2500, 1000, 500, 250, 100, 1];

/// MAZE BOUNDS CHECKING-------------------------------------------------------

pub fn reset_build(maze: &mut maze::Maze) {
    maze.as_slice_mut().fill(0b0)
}

pub fn choose_arbitrary_point(maze: &maze::Maze, parity: ParityPoint) -> Option<maze::Point> {
    let init = if parity == ParityPoint::Even { 2 } else { 1 };
    for r in (init..maze.row_size() - 1).step_by(2) {
        for c in (init..maze.col_size() - 1).step_by(2) {
            if (maze.get(r, c) & BUILDER_BIT) == 0 {
                return Some(maze::Point { row: r, col: c });
            }
        }
    }
    None
}

pub fn choose_point_from_row_start(
    maze: &maze::Maze,
    row_start: i32,
    parity: ParityPoint,
) -> Option<maze::Point> {
    let init = if parity == ParityPoint::Even { 2 } else { 1 };
    if (row_start % 2) != (init % 2) {
        maze_panic!("Row start parity did not match requested parity.");
    }
    for r in (row_start..maze.row_size() - 1).step_by(2) {
        for c in (init..maze.col_size() - 1).step_by(2) {
            if (maze.get(r, c) & BUILDER_BIT) == 0 {
                return Some(maze::Point { row: r, col: c });
            }
        }
    }
    None
}

#[inline]
pub fn can_build_new_square(maze: &maze::Maze, next: maze::Point) -> bool {
    next.row > 0
        && next.row < maze.row_size() - 1
        && next.col > 0
        && next.col < maze.col_size() - 1
        && (maze.get(next.row, next.col) & BUILDER_BIT) == 0
}

#[inline]
pub fn has_builder_bit(maze: &maze::Maze, next: maze::Point) -> bool {
    (maze.get(next.row, next.col) & BUILDER_BIT) != 0
}

#[inline]
pub fn is_built(square: maze::Square) -> bool {
    (square & BUILDER_BIT) != 0
}

#[inline]
pub fn is_square_within_perimeter_walls(maze: &maze::Maze, next: maze::Point) -> bool {
    next.row < maze.row_size() - 1 && next.row > 0 && next.col < maze.col_size() - 1 && next.col > 0
}

#[inline]
fn is_marked(square: maze::Square) -> bool {
    (square & MARKERS_MASK) != 0
}

#[inline]
fn get_mark(square: maze::Square) -> BacktrackSymbol {
    BACKTRACKING_SYMBOLS[((square & MARKERS_MASK) >> MARKER_SHIFT) as usize]
}

/// WALL ADDER HELPERS-------------------------------------------------------------------

pub fn build_wall_outline(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if c == 0 || c == maze.col_size() - 1 || r == 0 || r == maze.row_size() - 1 {
                *maze.get_mut(r, c) |= BUILDER_BIT;
                build_wall_carefully(maze, maze::Point { row: r, col: c });
                continue;
            }
            build_path(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn build_wall_outline_history(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if c == 0 || c == maze.col_size() - 1 || r == 0 || r == maze.row_size() - 1 {
                *maze.get_mut(r, c) |= BUILDER_BIT;
                build_wall_carefully(maze, maze::Point { row: r, col: c });
                continue;
            }
            build_path(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn build_wall_line(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        wall |= maze::NORTH_WALL;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
    }
    *maze.get_mut(p.row, p.col) |= wall | BUILDER_BIT;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
}

pub fn build_wall_line_history(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall_changes = [tape::Delta::default(); 5];
    let mut burst = 1;
    let mut wall: maze::WallLine = 0b0;
    let square = maze.get(p.row, p.col);
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        let neighbor = maze.get(p.row - 1, p.col);
        wall_changes[burst - 1] = tape::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: neighbor,
            after: (neighbor | maze::SOUTH_WALL),
            burst,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        let neighbor = maze.get(p.row + 1, p.col);
        wall_changes[burst - 1] = tape::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: neighbor,
            after: (neighbor | maze::NORTH_WALL),
            burst,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        let neighbor = maze.get(p.row, p.col - 1);
        wall_changes[burst - 1] = tape::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: neighbor,
            after: (neighbor | maze::EAST_WALL),
            burst,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        let neighbor = maze.get(p.row, p.col + 1);
        wall_changes[burst - 1] = tape::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: neighbor,
            after: (neighbor | maze::WEST_WALL),
            burst,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
        wall |= maze::EAST_WALL;
    }
    wall_changes[burst - 1] = tape::Delta {
        id: p,
        before: square,
        after: (square | wall | BUILDER_BIT) & !maze::PATH_BIT,
        burst,
    };
    *maze.get_mut(p.row, p.col) = (square | wall | BUILDER_BIT) & !maze::PATH_BIT;
    maze.build_history.push_burst(wall_changes.as_slice());
}

pub fn build_wall_line_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        wall |= maze::NORTH_WALL;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    *maze.get_mut(p.row, p.col) |= wall;
    *maze.get_mut(p.row, p.col) |= BUILDER_BIT;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
    flush_cursor_maze_coordinate(
        maze,
        maze::Point {
            row: p.row,
            col: p.col,
        },
    );
    thread::sleep(time::Duration::from_micros(speed));
}

pub fn build_mini_wall_line_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        wall |= maze::NORTH_WALL;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    *maze.get_mut(p.row, p.col) |= wall;
    *maze.get_mut(p.row, p.col) |= BUILDER_BIT;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
    flush_mini_backtracker_coordinate(
        maze,
        maze::Point {
            row: p.row,
            col: p.col,
        },
    );
    thread::sleep(time::Duration::from_micros(speed));
}

/// PATH CARVING HELPERS-------------------------------------------------------------------

pub fn fill_maze_history_with_walls(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            build_wall(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn fill_maze_with_walls(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            build_wall_history(maze, maze::Point { row: r, col: c });
        }
    }
    maze.build_history[0].burst = (maze.row_size() * maze.col_size()) as usize;
}

pub fn mark_origin(maze: &mut maze::Maze, walk: maze::Point, next: maze::Point) {
    if next.row > walk.row {
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
    } else if next.row < walk.row {
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
    } else if next.col < walk.col {
        *maze.get_mut(next.row, next.col) |= FROM_EAST;
    } else if next.col > walk.col {
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
    }
}

pub fn mark_origin_animated(
    maze: &mut maze::Maze,
    walk: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let mut wall = walk;
    if next.row > walk.row {
        wall.row += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_NORTH;
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
    } else if next.row < walk.row {
        wall.row -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_SOUTH;
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
    } else if next.col < walk.col {
        wall.col -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_EAST;
        *maze.get_mut(next.row, next.col) |= FROM_EAST;
    } else if next.col > walk.col {
        wall.col += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_WEST;
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
    }
    flush_cursor_maze_coordinate(maze, wall);
    thread::sleep(time::Duration::from_micros(speed));
    flush_cursor_maze_coordinate(maze, next);
    thread::sleep(time::Duration::from_micros(speed));
}

pub fn mark_mini_origin_animated(
    maze: &mut maze::Maze,
    walk: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let mut wall = walk;
    if next.row > walk.row {
        wall.row += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_NORTH;
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
    } else if next.row < walk.row {
        wall.row -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_SOUTH;
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
    } else if next.col < walk.col {
        wall.col -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_EAST;
        *maze.get_mut(next.row, next.col) |= FROM_EAST;
    } else if next.col > walk.col {
        wall.col += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_WEST;
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
    }
    flush_mini_backtracker_coordinate(maze, wall);
    thread::sleep(time::Duration::from_micros(speed));
    flush_mini_backtracker_coordinate(maze, next);
    thread::sleep(time::Duration::from_micros(speed));
}

pub fn carve_path_walls(maze: &mut maze::Maze, p: maze::Point) {
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT | BUILDER_BIT;
    if p.row > 0 {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
}

pub fn carve_wall_history(maze: &mut maze::Maze, p: maze::Point, backtracking: BacktrackMarker) {
    let mut wall_changes = [tape::Delta::default(); 5];
    let mut burst = 1;
    wall_changes[0] = tape::Delta {
        id: p,
        before: maze.get(p.row, p.col),
        after: maze.get(p.row, p.col) | maze::PATH_BIT | BUILDER_BIT | backtracking,
        burst,
    };
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT | BUILDER_BIT | backtracking;
    if p.row > 0 {
        wall_changes[burst] = tape::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: maze.get(p.row - 1, p.col),
            after: maze.get(p.row - 1, p.col) & !maze::SOUTH_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        wall_changes[burst] = tape::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: maze.get(p.row + 1, p.col),
            after: maze.get(p.row + 1, p.col) & !maze::NORTH_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        wall_changes[burst] = tape::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: maze.get(p.row, p.col - 1),
            after: maze.get(p.row, p.col - 1) & !maze::EAST_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        wall_changes[burst] = tape::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: maze.get(p.row, p.col + 1),
            after: maze.get(p.row, p.col + 1) & !maze::WEST_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    wall_changes[0].burst = burst;
    wall_changes[burst - 1].burst = burst;
    maze.build_history.push_burst(&wall_changes[0..burst]);
}

pub fn carve_path_walls_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT | BUILDER_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
}

pub fn carve_mini_walls_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT | BUILDER_BIT;
    print::set_cursor_position(
        maze::Point {
            row: (p.row) / 2,
            col: p.col,
        },
        maze.offset(),
    );
    flush_mini_backtracker_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
        flush_mini_backtracker_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
}

pub fn carve_path_markings(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    let mut wall: maze::Point = cur;
    if next.row < cur.row {
        wall.row -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_SOUTH;
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_NORTH;
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_EAST;
        *maze.get_mut(next.row, next.col) |= FROM_EAST;
    } else if next.col > cur.col {
        wall.col += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_WEST;
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    }
    carve_path_walls(maze, cur);
    carve_path_walls(maze, next);
    carve_path_walls(maze, wall);
}

pub fn carve_path_history(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    carve_wall_history(maze, cur, maze.get(cur.row, cur.col) & MARKERS_MASK);
    let mut wall: maze::Point = cur;
    let backtracking = if next.row < cur.row {
        wall.row -= 1;
        FROM_SOUTH
    } else if next.row > cur.row {
        wall.row += 1;
        FROM_NORTH
    } else if next.col < cur.col {
        wall.col -= 1;
        FROM_EAST
    } else if next.col > cur.col {
        wall.col += 1;
        FROM_WEST
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    };
    carve_wall_history(maze, wall, backtracking);
    carve_wall_history(maze, next, backtracking);
}

pub fn carve_path_markings_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let mut wall: maze::Point = cur;
    if next.row < cur.row {
        wall.row -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_SOUTH;
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_NORTH;
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_EAST;
        *maze.get_mut(next.row, next.col) |= FROM_EAST;
    } else if next.col > cur.col {
        wall.col += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_WEST;
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    }
    carve_path_walls_animated(maze, cur, speed);
    carve_path_walls_animated(maze, next, speed);
    carve_path_walls_animated(maze, wall, speed);
}

pub fn carve_mini_markings_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let mut wall: maze::Point = cur;
    if next.row < cur.row {
        wall.row -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_SOUTH;
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_NORTH;
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_EAST;
        *maze.get_mut(next.row, next.col) |= FROM_EAST
    } else if next.col > cur.col {
        wall.col += 1;
        *maze.get_mut(wall.row, wall.col) |= FROM_WEST;
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    }
    carve_mini_walls_animated(maze, cur, speed);
    carve_mini_walls_animated(maze, next, speed);
    carve_mini_walls_animated(maze, wall, speed);
}

pub fn join_squares(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    build_path(maze, cur);
    *maze.get_mut(cur.row, cur.col) |= BUILDER_BIT;
    let mut wall = cur;
    if next.row < cur.row {
        wall.row -= 1;
    } else if next.row > cur.row {
        wall.row += 1;
    } else if next.col < cur.col {
        wall.col -= 1;
    } else if next.col > cur.col {
        wall.col += 1;
    } else {
        print::maze_panic!("Cell join error. Cur: {:?} Next: {:?}", cur, next);
    }
    build_path(maze, wall);
    *maze.get_mut(wall.row, wall.col) |= BUILDER_BIT;
    build_path(maze, next);
    *maze.get_mut(next.row, next.col) |= BUILDER_BIT;
}

pub fn join_squares_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let mut wall = cur;
    if next.row < cur.row {
        wall.row -= 1;
    } else if next.row > cur.row {
        wall.row += 1;
    } else if next.col < cur.col {
        wall.col -= 1;
    } else if next.col > cur.col {
        wall.col += 1;
    } else {
        print::maze_panic!("Cell join error. Maze won't build");
    }
    carve_path_walls_animated(maze, cur, speed);
    carve_path_walls_animated(maze, wall, speed);
    carve_path_walls_animated(maze, next, speed);
}

pub fn join_minis_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let mut wall = cur;
    if next.row < cur.row {
        wall.row -= 1;
    } else if next.row > cur.row {
        wall.row += 1;
    } else if next.col < cur.col {
        wall.col -= 1;
    } else if next.col > cur.col {
        wall.col += 1;
    } else {
        print::maze_panic!("Cell join error. Maze won't build");
    }
    carve_mini_walls_animated(maze, cur, speed);
    carve_mini_walls_animated(maze, next, speed);
    carve_mini_walls_animated(maze, wall, speed);
}

pub fn build_wall_history(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 {
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 {
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        wall |= maze::EAST_WALL;
    }
    maze.build_history.push(tape::Delta {
        id: p,
        before: 0b0,
        after: wall & !maze::PATH_BIT,
        burst: 1,
    });
    *maze.get_mut(p.row, p.col) |= wall;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
}

pub fn build_wall(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 {
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 {
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        wall |= maze::EAST_WALL;
    }
    *maze.get_mut(p.row, p.col) |= wall;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
}

pub fn build_wall_carefully(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        wall |= maze::NORTH_WALL;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
    }
    *maze.get_mut(p.row, p.col) |= wall;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
}

pub fn build_path(maze: &mut maze::Maze, p: maze::Point) {
    if p.row > 0 {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT;
}

pub fn build_path_history(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall_changes = [tape::Delta::default(); 5];
    let mut burst = 1;
    let mut square = maze.get(p.row, p.col);
    wall_changes[0] = tape::Delta {
        id: p,
        before: square,
        after: square | maze::PATH_BIT,
        burst,
    };
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT;
    if p.row > 0 {
        burst += 1;
        square = maze.get(p.row - 1, p.col);
        wall_changes[0] = tape::Delta {
            id: p,
            before: square,
            after: square & !maze::SOUTH_WALL,
            burst,
        };
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        burst += 1;
        square = maze.get(p.row + 1, p.col);
        wall_changes[0] = tape::Delta {
            id: p,
            before: square,
            after: square & !maze::NORTH_WALL,
            burst,
        };
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        burst += 1;
        square = maze.get(p.row, p.col - 1);
        wall_changes[0] = tape::Delta {
            id: p,
            before: square,
            after: square & !maze::EAST_WALL,
            burst,
        };
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        burst += 1;
        square = maze.get(p.row, p.col + 1);
        wall_changes[0] = tape::Delta {
            id: p,
            before: square,
            after: square & !maze::WEST_WALL,
            burst,
        };
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    wall_changes[0].burst = burst;
    maze.build_history.push_burst(wall_changes.as_slice());
}

pub fn build_path_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
}

pub fn build_mini_path_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    *maze.get_mut(p.row, p.col) |= maze::PATH_BIT;
    flush_mini_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
        flush_mini_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && maze.wall_at(p.row + 1, p.col) {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
        flush_mini_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
        flush_mini_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && maze.wall_at(p.row, p.col + 1) {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
        flush_mini_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
}

pub fn print_overlap_key(maze: &maze::Maze) {
    let offset = maze::Offset {
        add_rows: if maze.style_index() == (maze::MazeStyle::Mini as usize) {
            maze.offset().add_rows + maze.row_size() / 2 + 1
        } else {
            maze.offset().add_rows + maze.row_size()
        },
        add_cols: maze.offset().add_cols,
    };
    let mut cur_print = 0;
    for r in 0..THREAD_KEY_MAZE_ROWS {
        for c in (0..THREAD_KEY_MAZE_COLS).step_by((KEY_ENTRY_LEN) as usize) {
            let cur_pos = maze::Point { row: r, col: c };
            print::set_cursor_position(cur_pos, offset);
            let thread_info = key::thread_color(cur_print);
            execute!(
                io::stdout(),
                SetForegroundColor(Color::AnsiValue(thread_info.ansi)),
                Print(format!("{}{:04b}", thread_info.block, cur_print)),
                ResetColor,
            )
            .expect("Could not execute print_overlap_key_command");
            cur_print += 1;
        }
    }
    print::set_cursor_position(maze::Point { row: 0, col: 0 }, offset);
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Grey),
        Print(format!(" {:04b}", 0)),
        ResetColor,
    )
    .expect("Could not execute print_overlap_key_command");
}

pub fn print_overlap_key_animated(maze: &maze::Maze) {
    let offset = maze::Offset {
        add_rows: if maze.style_index() == (maze::MazeStyle::Mini as usize) {
            maze.offset().add_rows + maze.row_size() / 2 + 1
        } else {
            maze.offset().add_rows + maze.row_size()
        },
        add_cols: maze.offset().add_cols,
    };
    let mut cur_print = 0;
    for r in 0..THREAD_KEY_MAZE_ROWS {
        for c in (0..THREAD_KEY_MAZE_COLS).step_by((KEY_ENTRY_LEN) as usize) {
            let cur_pos = maze::Point { row: r, col: c };
            print::set_cursor_position(cur_pos, offset);
            let thread_info = key::thread_color(cur_print);
            execute!(
                io::stdout(),
                SetForegroundColor(Color::AnsiValue(thread_info.ansi)),
                Print(format!("{}{:04b}", thread_info.block, cur_print)),
                ResetColor,
            )
            .expect("Could not execute print_overlap_key_command");
            cur_print += 1;
        }
    }
    print::set_cursor_position(maze::Point { row: 0, col: 0 }, offset);
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Grey),
        Print(format!(" {:04b}", 0)),
        ResetColor,
    )
    .expect("Could not execute print_overlap_key_command");
}

// Terminal Printing Helpers

pub fn decode_square(wall_row: &[char], square: maze::Square) -> Cell {
    if is_marked(square) {
        let mark = get_mark(square);
        Cell {
            symbol: mark.arrow.to_string(),
            fg: RatColor::Indexed(ANSI_WHITE),
            bg: RatColor::Indexed(mark.ansi),
            underline_color: RatColor::Reset,
            modifier: Modifier::BOLD,
            skip: false,
        }
    } else if maze::is_wall(square) {
        Cell {
            symbol: wall_row[(square & maze::WALL_MASK) as usize].to_string(),
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
        print::maze_panic!("Printed a maze square without a category.");
    }
}

pub fn flush_cursor_maze_coordinate(maze: &maze::Maze, p: maze::Point) {
    let square = maze.get(p.row, p.col);
    print::set_cursor_position(p, maze.offset());
    if is_marked(square) {
        let mark = get_mark(square);
        execute!(
            io::stdout(),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(ANSI_WHITE)),
            SetBackgroundColor(Color::AnsiValue(mark.ansi)),
            Print(mark.arrow),
            ResetColor,
        )
        .expect("Could not print backtrack point.");
    } else if maze::is_wall(square) {
        execute!(io::stdout(), Print(maze.wall_char(square)),).expect("Could not print wall.");
    } else if maze::is_path(square) {
        execute!(io::stdout(), Print(' '),).expect("Could not print path");
    } else {
        print::maze_panic!("Printed a maze square without a category.");
    }
}

pub fn print_square(maze: &maze::Maze, p: maze::Point) {
    let square = maze.get(p.row, p.col);
    print::set_cursor_position(p, maze.offset());
    if square & MARKERS_MASK != 0 {
        let mark = BACKTRACKING_SYMBOLS[((square & MARKERS_MASK) >> MARKER_SHIFT) as usize];
        queue!(
            io::stdout(),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(ANSI_WHITE)),
            SetBackgroundColor(Color::AnsiValue(mark.ansi)),
            Print(mark.arrow),
            ResetColor,
        )
        .expect("Could not print backtrack point.");
    } else if maze::is_wall(square) {
        queue!(io::stdout(), Print(maze.wall_char(square)),).expect("Could not print wall.");
    } else if maze::is_path(square) {
        queue!(io::stdout(), Print(' '),).expect("Could not print path");
    } else {
        print::maze_panic!("Printed a maze square without a category.");
    }
}

pub fn flush_mini_backtracker_coordinate(maze: &maze::Maze, p: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: p.row / 2,
            col: p.col,
        },
        maze.offset(),
    );
    let square = maze.get(p.row, p.col);
    if maze::is_wall(square) {
        // Need this for wilson backtracking while random walking.
        if square & MARKERS_MASK != 0 {
            execute!(
                io::stdout(),
                SetBackgroundColor(Color::AnsiValue(
                    BACKTRACKING_SYMBOLS[((square & MARKERS_MASK) >> MARKER_SHIFT) as usize].ansi
                )),
                Print('▀'),
                ResetColor
            )
            .expect("Could not print.");
            return;
        }
        let mut color = 0;
        if p.row % 2 != 0 && p.row > 0 {
            color = BACKTRACKING_SYMBOLS
                [((maze.get(p.row - 1, p.col) & MARKERS_MASK) >> MARKER_SHIFT) as usize]
                .ansi;
        } else if p.row + 1 < maze.row_size() {
            color = BACKTRACKING_SYMBOLS
                [((maze.get(p.row + 1, p.col) & MARKERS_MASK) >> MARKER_SHIFT) as usize]
                .ansi;
        }
        execute!(
            io::stdout(),
            SetBackgroundColor(Color::AnsiValue(color)),
            Print(maze.wall_char(square)),
            ResetColor
        )
        .expect("Could not print.");
        return;
    }
    // We know this is a path but because we are half blocks we need to render correctly.
    if p.row % 2 == 0 {
        execute!(
            io::stdout(),
            SetBackgroundColor(Color::AnsiValue(
                BACKTRACKING_SYMBOLS[((square & MARKERS_MASK) >> MARKER_SHIFT) as usize].ansi
            )),
            Print(match maze.wall_at(p.row + 1, p.col) {
                true => '▄',
                false => ' ',
            }),
            ResetColor
        )
        .expect("Could not print.");
        return;
    }
    execute!(
        io::stdout(),
        SetBackgroundColor(Color::AnsiValue(
            BACKTRACKING_SYMBOLS[((square & MARKERS_MASK) >> MARKER_SHIFT) as usize].ansi
        )),
        Print(match maze.wall_at(p.row - 1, p.col) {
            true => '▀',
            false => ' ',
        }),
        ResetColor
    )
    .expect("Could not print.");
}

pub fn flush_mini_coordinate(maze: &maze::Maze, p: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: p.row / 2,
            col: p.col,
        },
        maze.offset(),
    );
    let square = maze.get(p.row, p.col);
    if maze::is_wall(square) {
        if p.row % 2 != 0 {
            // By defenition of 0 indexing row - 1 is safe now.
            execute!(
                io::stdout(),
                Print(maze.wall_char(maze.get(p.row - 1, p.col))),
                ResetColor
            )
            .expect("Could not print.");
            return;
        }
        execute!(io::stdout(), Print(maze.wall_char(square)), ResetColor)
            .expect("Could not print.");
        return;
    }
    if p.row % 2 == 0 {
        execute!(
            io::stdout(),
            Print(match maze.wall_at(p.row + 1, p.col) {
                true => '▄',
                false => ' ',
            }),
            ResetColor
        )
        .expect("Could not print.");
        return;
    }
    execute!(
        io::stdout(),
        Print(match p.row > 0 && maze.wall_at(p.row - 1, p.col) {
            true => '▀',
            false => ' ',
        }),
        ResetColor
    )
    .expect("Could not print.");
}

pub fn print_mini_coordinate(maze: &maze::Maze, p: maze::Point) {
    print::set_cursor_position(
        maze::Point {
            row: p.row / 2,
            col: p.col,
        },
        maze.offset(),
    );
    let square = maze.get(p.row, p.col);
    if maze::is_wall(square) {
        if p.row % 2 != 0 {
            // By defenition of 0 indexing row - 1 is safe now.
            queue!(
                io::stdout(),
                Print(maze.wall_char(maze.get(p.row - 1, p.col))),
                ResetColor
            )
            .expect("Could not print.");
            return;
        }
        queue!(io::stdout(), Print(maze.wall_char(square)), ResetColor).expect("Could not print.");
        return;
    }
    if p.row % 2 == 0 {
        queue!(
            io::stdout(),
            Print(match maze.wall_at(p.row + 1, p.col) {
                true => '▄',
                false => ' ',
            }),
            ResetColor
        )
        .expect("Could not print.");
        return;
    }
    queue!(
        io::stdout(),
        Print(match p.row > 0 && maze.wall_at(p.row - 1, p.col) {
            true => '▀',
            false => ' ',
        }),
        ResetColor
    )
    .expect("Could not print.");
}

pub fn flush_grid(maze: &maze::Maze) {
    if maze.style_index() == (maze::MazeStyle::Mini as usize) {
        for r in 0..maze.row_size() {
            for c in 0..maze.col_size() {
                print_mini_coordinate(maze, maze::Point { row: r, col: c });
            }
            match queue!(io::stdout(), Print('\n')) {
                Ok(_) => {}
                Err(_) => maze_panic!("Couldn't print square."),
            };
        }
    } else {
        for r in 0..maze.row_size() {
            for c in 0..maze.col_size() {
                print_square(maze, maze::Point { row: r, col: c });
            }
            match queue!(io::stdout(), Print('\n')) {
                Ok(_) => {}
                Err(_) => maze_panic!("Couldn't print square."),
            };
        }
    }
    print::flush();
}

// Debug function

pub fn flush_bit_vals(maze: &maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            let square = maze.get(r, c);
            eprint!(
                "{},{:2}|",
                match square & maze::PATH_BIT != 0 {
                    true => 1,
                    false => 0,
                },
                square & maze::WALL_MASK
            );
        }
        eprintln!();
    }
}

const KEY_ENTRY_LEN: i32 = 5;
const THREAD_KEY_MAZE_ROWS: i32 = 2;
const THREAD_KEY_MAZE_COLS: i32 = 8 * KEY_ENTRY_LEN;
