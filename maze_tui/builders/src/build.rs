use maze;
use print;
use print::maze_panic;
use ratatui::{
    buffer::Cell,
    style::{Color as RatColor, Modifier},
};

pub type SpeedUnit = u64;
pub type BacktrackMarker = u32;

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
pub const BUILDER_BIT: maze::Square = 0b0001_0000_0000_0000_0000_0000_0000_0000;
// Data that will help backtracker algorithms like recursive backtracker and Wilson's.
pub const NUM_DIRECTIONS: usize = 4;
pub const MARKERS_MASK: BacktrackMarker = 0b1111;
pub const FROM_NORTH: BacktrackMarker = 0b0001;
pub const FROM_EAST: BacktrackMarker = 0b0010;
pub const FROM_SOUTH: BacktrackMarker = 0b0011;
pub const FROM_WEST: BacktrackMarker = 0b0100;
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

// MAZE BOUNDS CHECKING AND HELPERS-------------------------------------------------------

pub fn reset_build(maze: &mut maze::Maze) {
    maze.as_slice_mut().fill(0b0)
}

pub fn choose_arbitrary_point(maze: &maze::Maze, parity: ParityPoint) -> Option<maze::Point> {
    let init = if parity == ParityPoint::Even { 2 } else { 1 };
    for r in (init..maze.rows() - 1).step_by(2) {
        for c in (init..maze.cols() - 1).step_by(2) {
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
    for r in (row_start..maze.rows() - 1).step_by(2) {
        for c in (init..maze.cols() - 1).step_by(2) {
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
        && next.row < maze.rows() - 1
        && next.col > 0
        && next.col < maze.cols() - 1
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
    next.row < maze.rows() - 1 && next.row > 0 && next.col < maze.cols() - 1 && next.col > 0
}

#[inline]
pub fn is_marked(square: maze::Square) -> bool {
    (square & MARKERS_MASK) != 0
}

#[inline]
fn get_mark(square: maze::Square) -> BacktrackSymbol {
    BACKTRACKING_SYMBOLS[(square & MARKERS_MASK) as usize]
}

// WALL ADDER HELPERS-------------------------------------------------------------------

// Data Driven functions no IO or animation concerns.

pub fn build_wall_outline(maze: &mut maze::Maze) {
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
            if c == 0 || c == maze.cols() - 1 || r == 0 || r == maze.rows() - 1 {
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
    if p.row + 1 < maze.rows() && maze.wall_at(p.row + 1, p.col) {
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() && maze.wall_at(p.row, p.col + 1) {
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
    }
    *maze.get_mut(p.row, p.col) |= wall | BUILDER_BIT;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
}

pub fn carve_path_walls(maze: &mut maze::Maze, p: maze::Point) {
    let square = maze.get(p.row, p.col);
    *maze.get_mut(p.row, p.col) = (square & !maze::WALL_MASK) | maze::PATH_BIT | BUILDER_BIT;
    if p.row > 0 {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
}

// PATH CARVING HELPERS-------------------------------------------------------------------

///
/// Data Driven with no IO or animation.
///
pub fn fill_maze_with_walls(maze: &mut maze::Maze) {
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
            build_wall(maze, maze::Point { row: r, col: c });
        }
    }
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
    } else {
        print::maze_panic!("next cannot be equal to walk");
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

pub fn build_path(maze: &mut maze::Maze, p: maze::Point) {
    if p.row > 0 {
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() {
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    let square = maze.get(p.row, p.col);
    *maze.get_mut(p.row, p.col) = (square & !maze::WALL_MASK) | maze::PATH_BIT;
}

pub fn build_wall(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 {
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 {
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.cols() {
        wall |= maze::EAST_WALL;
    }
    *maze.get_mut(p.row, p.col) = wall;
}

pub fn build_wall_carefully(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        wall |= maze::NORTH_WALL;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() && maze.wall_at(p.row + 1, p.col) {
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() && maze.wall_at(p.row, p.col + 1) {
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
    }
    *maze.get_mut(p.row, p.col) |= wall;
    *maze.get_mut(p.row, p.col) &= !maze::PATH_BIT;
}

///
/// History tracking for playback and animation.
///
pub fn fill_maze_history_with_walls(maze: &mut maze::Maze) {
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
            build_wall_history(maze, maze::Point { row: r, col: c });
        }
    }
    let burst = (maze.rows() * maze.cols()) as usize;
    maze.build_history[0].burst = burst;
    maze.build_history[burst - 1].burst = burst;
}

pub fn mark_origin_history(maze: &mut maze::Maze, walk: maze::Point, next: maze::Point) {
    let mut wall = walk;
    let next_before = maze.get(next.row, next.col);
    let wall_before = if next.row > walk.row {
        wall.row += 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) |= FROM_NORTH;
        *maze.get_mut(next.row, next.col) |= FROM_NORTH;
        before
    } else if next.row < walk.row {
        wall.row -= 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) |= FROM_SOUTH;
        *maze.get_mut(next.row, next.col) |= FROM_SOUTH;
        before
    } else if next.col < walk.col {
        wall.col -= 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) |= FROM_EAST;
        *maze.get_mut(next.row, next.col) |= FROM_EAST;
        before
    } else if next.col > walk.col {
        wall.col += 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) |= FROM_WEST;
        *maze.get_mut(next.row, next.col) |= FROM_WEST;
        before
    } else {
        print::maze_panic!("next cannot be equal to walk");
    };
    maze.build_history.push(maze::Delta {
        id: wall,
        before: wall_before,
        after: maze.get(wall.row, wall.col),
        burst: 1,
    });
    maze.build_history.push(maze::Delta {
        id: next,
        before: next_before,
        after: maze.get(next.row, next.col),
        burst: 1,
    });
}

pub fn build_wall_outline_history(maze: &mut maze::Maze) {
    let mut deltas = 0;
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
            if c == 0 || c == maze.cols() - 1 || r == 0 || r == maze.rows() - 1 {
                *maze.get_mut(r, c) |= BUILDER_BIT;
                deltas += build_wall_history_carefully(maze, maze::Point { row: r, col: c });
                continue;
            }
            deltas += build_path_history(maze, maze::Point { row: r, col: c });
        }
    }
    maze.build_history[0].burst = deltas;
    maze.build_history[deltas - 1].burst = deltas;
}

pub fn build_wall_line_history(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall_changes = [maze::Delta::default(); 5];
    let mut burst = 1;
    let mut wall: maze::WallLine = 0b0;
    let square = maze.get(p.row, p.col);
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        let neighbor = maze.get(p.row - 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: neighbor,
            after: (neighbor | maze::SOUTH_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.rows() && maze.wall_at(p.row + 1, p.col) {
        let neighbor = maze.get(p.row + 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: neighbor,
            after: (neighbor | maze::NORTH_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        let neighbor = maze.get(p.row, p.col - 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: neighbor,
            after: (neighbor | maze::EAST_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.cols() && maze.wall_at(p.row, p.col + 1) {
        let neighbor = maze.get(p.row, p.col + 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: neighbor,
            after: (neighbor | maze::WEST_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
        wall |= maze::EAST_WALL;
    }
    wall_changes[0] = maze::Delta {
        id: p,
        before: square,
        after: (square | wall | BUILDER_BIT) & !maze::PATH_BIT,
        burst,
    };
    *maze.get_mut(p.row, p.col) = (square | wall | BUILDER_BIT) & !maze::PATH_BIT;
    maze.build_history.push_burst(&wall_changes[0..burst]);
}

pub fn carve_wall_history(maze: &mut maze::Maze, p: maze::Point, backtracking: BacktrackMarker) {
    let mut wall_changes = [maze::Delta::default(); 5];
    let mut burst = 1;
    let before = maze.get(p.row, p.col);
    wall_changes[0] = maze::Delta {
        id: p,
        before,
        after: (before & !maze::WALL_MASK) | maze::PATH_BIT | BUILDER_BIT | backtracking,
        burst,
    };
    *maze.get_mut(p.row, p.col) =
        (before & !maze::WALL_MASK) | maze::PATH_BIT | BUILDER_BIT | backtracking;
    if p.row > 0 {
        let square = maze.get(p.row - 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: square,
            after: square & !maze::SOUTH_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        let square = maze.get(p.row + 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: square,
            after: square & !maze::NORTH_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        let square = maze.get(p.row, p.col - 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: square,
            after: square & !maze::EAST_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() {
        let square = maze.get(p.row, p.col + 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: square,
            after: square & !maze::WEST_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    wall_changes[0].burst = burst;
    maze.build_history.push_burst(&wall_changes[0..burst]);
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

pub fn join_squares_history(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    build_path_history(maze, cur);
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
    build_path_history(maze, wall);
    *maze.get_mut(wall.row, wall.col) |= BUILDER_BIT;
    build_path_history(maze, next);
    *maze.get_mut(next.row, next.col) |= BUILDER_BIT;
}

pub fn build_wall_history(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 {
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 {
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.cols() {
        wall |= maze::EAST_WALL;
    }
    maze.build_history.push(maze::Delta {
        id: p,
        before: 0b0,
        after: wall,
        burst: 1,
    });
    *maze.get_mut(p.row, p.col) = wall;
}

pub fn build_wall_history_carefully(maze: &mut maze::Maze, p: maze::Point) -> usize {
    let mut deltas = [maze::Delta::default(); 5];
    let mut burst = 1;
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        let square = maze.get(p.row - 1, p.col);
        deltas[burst] = maze::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: square,
            after: square | maze::SOUTH_WALL,
            burst: 1,
        };
        burst += 1;
        wall |= maze::NORTH_WALL;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() && maze.wall_at(p.row + 1, p.col) {
        let square = maze.get(p.row + 1, p.col);
        deltas[burst] = maze::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: square,
            after: square | maze::NORTH_WALL,
            burst: 1,
        };
        burst += 1;
        wall |= maze::SOUTH_WALL;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        let square = maze.get(p.row, p.col - 1);
        deltas[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: square,
            after: square | maze::EAST_WALL,
            burst: 1,
        };
        burst += 1;
        wall |= maze::WEST_WALL;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() && maze.wall_at(p.row, p.col + 1) {
        let square = maze.get(p.row, p.col + 1);
        deltas[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: square,
            after: square | maze::WEST_WALL,
            burst: 1,
        };
        burst += 1;
        wall |= maze::EAST_WALL;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
    }
    let before = maze.get(p.row, p.col);
    deltas[0] = maze::Delta {
        id: p,
        before,
        after: before | wall,
        burst,
    };
    deltas[burst - 1].burst = burst;
    *maze.get_mut(p.row, p.col) |= (before & !maze::PATH_BIT) | wall;
    maze.build_history.push_burst(&deltas[0..burst]);
    burst
}

pub fn build_path_history(maze: &mut maze::Maze, p: maze::Point) -> usize {
    let mut wall_changes = [maze::Delta::default(); 5];
    let mut burst = 1;
    let mut square = maze.get(p.row, p.col);
    wall_changes[0] = maze::Delta {
        id: p,
        before: square,
        after: (square & !maze::WALL_MASK) | maze::PATH_BIT,
        burst,
    };
    *maze.get_mut(p.row, p.col) = (square & !maze::WALL_MASK) | maze::PATH_BIT;
    if p.row > 0 {
        square = maze.get(p.row - 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: square,
            after: square & !maze::SOUTH_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        square = maze.get(p.row + 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: square,
            after: square & !maze::NORTH_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        square = maze.get(p.row, p.col - 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: square,
            after: square & !maze::EAST_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() {
        square = maze.get(p.row, p.col + 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: square,
            after: square & !maze::WEST_WALL,
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    wall_changes[0].burst = burst;
    maze.build_history.push_burst(&wall_changes[0..burst]);
    burst
}

///
/// Display Code: This section is concerned with the lower level details of how elements are
/// displayed to the screen. For the printers that means interacting with Crossterm's print
/// and cursor movement capabilities. For the TUI that means choosing how to decode every possible
/// maze square and display it as a ratatui.rs buffer Cell.
///
///
/// TUI rendering helpers for decoding maze squares into Cells.
///
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
        print::maze_panic!("Printed a maze square without a category.");
    }
}

pub fn decode_mini_square(maze: &maze::Blueprint, p: maze::Point) -> Cell {
    let square = maze.get(p.row, p.col);
    if maze::is_wall(square) {
        // Need this for wilson backtracking while random walking.
        if is_marked(square) {
            if p.row % 2 == 0 {
                let fg = match get_mark(maze.get(p.row + 1, p.col)) {
                    BacktrackSymbol {
                        arrow: ' ',
                        ansi: 0,
                    } => RatColor::Reset,
                    any => RatColor::Indexed(any.ansi),
                };
                return Cell {
                    symbol: '▀'.to_string(),
                    fg,
                    bg: RatColor::Indexed(get_mark(square).ansi),
                    underline_color: RatColor::Reset,
                    modifier: Modifier::empty(),
                    skip: false,
                };
            }
            let fg = match get_mark(maze.get(p.row - 1, p.col)) {
                BacktrackSymbol {
                    arrow: ' ',
                    ansi: 0,
                } => RatColor::Reset,
                any => RatColor::Indexed(any.ansi),
            };
            return Cell {
                symbol: '▀'.to_string(),
                fg,
                bg: RatColor::Indexed(get_mark(square).ansi),
                underline_color: RatColor::Reset,
                modifier: Modifier::empty(),
                skip: false,
            };
        }
        let mut color = 0;
        if p.row % 2 != 0 && p.row > 0 {
            color = get_mark(maze.get(p.row - 1, p.col)).ansi;
        } else if p.row + 1 < maze.rows {
            color = get_mark(maze.get(p.row + 1, p.col)).ansi;
        }
        return Cell {
            symbol: maze.wall_char(square).to_string(),
            fg: RatColor::Reset,
            bg: RatColor::Indexed(color),
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        };
    }
    // We know this is a path but because we are half blocks we need to render correctly.
    if p.row % 2 == 0 {
        let fg = match get_mark(maze.get(p.row + 1, p.col)) {
            BacktrackSymbol {
                arrow: ' ',
                ansi: 0,
            } => RatColor::Reset,
            any => RatColor::Indexed(any.ansi),
        };
        return Cell {
            symbol: match maze.wall_at(p.row + 1, p.col) {
                true => '▄',
                false => ' ',
            }
            .to_string(),
            fg,
            bg: RatColor::Indexed(get_mark(square).ansi),
            underline_color: RatColor::Reset,
            modifier: Modifier::empty(),
            skip: false,
        };
    }
    let fg = match get_mark(maze.get(p.row - 1, p.col)) {
        BacktrackSymbol {
            arrow: ' ',
            ansi: 0,
        } => RatColor::Reset,
        any => RatColor::Indexed(any.ansi),
    };
    Cell {
        symbol: match maze.wall_at(p.row - 1, p.col) {
            true => '▀',
            false => ' ',
        }
        .to_string(),
        fg,
        bg: RatColor::Indexed(get_mark(square).ansi),
        underline_color: RatColor::Reset,
        modifier: Modifier::empty(),
        skip: false,
    }
}

// Debug function

pub fn flush_bit_vals(maze: &maze::Maze) {
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
            let square = maze.get(r, c);
            eprint!(
                "{},{:2}|",
                match square & maze::PATH_BIT != 0 {
                    true => 1,
                    false => 0,
                },
                (square & maze::WALL_MASK) >> maze::WALL_SHIFT
            );
        }
        eprintln!();
    }
}
