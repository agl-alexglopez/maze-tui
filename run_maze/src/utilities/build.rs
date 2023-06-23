use crate::maze;
use crate::utilities::print;

use std::{thread, time};

pub type SpeedUnit = u64;
pub type BacktrackMarker = u16;

#[derive(Clone, Copy)]
pub enum BuilderSpeed {
    Instant = 0,
    Speed1,
    Speed2,
    Speed3,
    Speed4,
    Speed5,
    Speed6,
    Speed7,
}

#[derive(PartialEq, Eq)]
pub enum ParityPoint {
    Even,
    Odd,
}

// Data that will help backtracker algorithms like recursive backtracker and Wilson's.
pub const MARKER_SHIFT: u8 = 4;
pub const NUM_DIRECTIONS: usize = 4;
pub const MARKERS_MASK: BacktrackMarker = 0b1111_0000;
pub const IS_ORIGIN: BacktrackMarker = 0b0000_0000;
pub const FROM_NORTH: BacktrackMarker = 0b0001_0000;
pub const FROM_EAST: BacktrackMarker = 0b0010_0000;
pub const FROM_SOUTH: BacktrackMarker = 0b0011_0000;
pub const FROM_WEST: BacktrackMarker = 0b0100_0000;
pub static BACKTRACKING_SYMBOLS: [&str; 5] = [
    " ",                                 // I came from the orgin.
    "\x1b[38;5;15m\x1b[48;5;1m↑\x1b[0m", // I came from the north.
    "\x1b[38;5;15m\x1b[48;5;2m→\x1b[0m", // I came from the east.
    "\x1b[38;5;15m\x1b[48;5;3m↓\x1b[0m", // I came from the south.
    "\x1b[38;5;15m\x1b[48;5;4m←\x1b[0m", // I came from the west.
];
pub const BACKTRACKING_POINTS: [maze::Point; 5] = [
    maze::Point { row: 0, col: 0 },
    maze::Point { row: -2, col: 0 },
    maze::Point { row: 0, col: 2 },
    maze::Point { row: 2, col: 0 },
    maze::Point { row: 0, col: -2 },
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

// Maze Modification Helpers

pub fn add_positive_slope(maze: &mut maze::Maze, p: maze::Point) {
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.col_size() - 2 && p.col > 1 {
        build_path(maze, p);
        if p.col + 1 < maze.col_size() - 2 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
            );
        }
        if p.col - 1 > 1 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
            );
        }
        if p.col + 2 < maze.col_size() - 2 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
            );
        }
        if p.col - 2 > 1 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
            );
        }
    }
}

pub fn add_positive_slope_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.col_size() - 2 && p.col > 1 {
        build_path_animated(maze, p, speed);
        if p.col + 1 < maze.col_size() - 2 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
                speed,
            );
        }
        if p.col - 1 > 1 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
                speed,
            );
        }
        if p.col + 2 < maze.col_size() - 2 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
                speed,
            );
        }
        if p.col - 2 > 1 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
                speed,
            );
        }
    }
}

pub fn add_negative_slope(maze: &mut maze::Maze, p: maze::Point) {
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.col_size() - 2 && p.row < maze.row_size() - 2 {
        build_path(maze, p);
        if p.col + 1 < maze.col_size() - 2 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
            );
        }
        if p.col - 1 > 1 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
            );
        }
        if p.col + 2 < maze.col_size() - 2 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
            );
        }
        if p.col - 2 > 1 {
            build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
            );
        }
    }
}

pub fn add_negative_slope_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.col_size() - 2 && p.row < maze.row_size() - 2 {
        build_path_animated(maze, p, speed);
        if p.col + 1 < maze.col_size() - 2 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
                speed,
            );
        }
        if p.col - 1 > 1 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
                speed,
            );
        }
        if p.col + 2 < maze.col_size() - 2 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
                speed,
            );
        }
        if p.col - 2 > 1 {
            build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
                speed,
            );
        }
    }
}

pub fn add_cross(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if (r == maze.row_size() / 2 && c > 1 && c < maze.col_size() - 2)
                || (c == maze.col_size() / 2 && r > 1 && r < maze.row_size() - 2)
            {
                build_path(maze, maze::Point { row: r, col: c });
                if c + 1 < maze.col_size() - 2 {
                    build_path(maze, maze::Point { row: r, col: c + 1 });
                }
            }
        }
    }
}

pub fn add_cross_animated(maze: &mut maze::Maze, speed: BuilderSpeed) {
    let animation = BUILDER_SPEEDS[speed as usize];
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if (r == maze.row_size() / 2 && c > 1 && c < maze.col_size() - 2)
                || (c == maze.col_size() / 2 && r > 1 && r < maze.row_size() - 2)
            {
                build_path_animated(maze, maze::Point { row: r, col: c }, animation);
                if c + 1 < maze.col_size() - 2 {
                    build_path_animated(maze, maze::Point { row: r, col: c + 1 }, animation);
                }
            }
        }
    }
}

pub fn add_x(maze: &mut maze::Maze) {
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            add_positive_slope(maze, maze::Point { row: r, col: c });
            add_negative_slope(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn add_x_animated(maze: &mut maze::Maze, speed: BuilderSpeed) {
    let animation: SpeedUnit = BUILDER_SPEEDS[speed as usize];
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            add_positive_slope_animated(maze, maze::Point { row: r, col: c }, animation);
            add_negative_slope_animated(maze, maze::Point { row: r, col: c }, animation);
        }
    }
}

pub fn build_wall_outline(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if c == 0 || c == maze.col_size() - 1 || r == 0 || r == maze.row_size() - 1 {
                maze[r as usize][c as usize] |= maze::BUILDER_BIT;
                build_wall_carefully(maze, maze::Point { row: r, col: c });
                continue;
            }
            build_path(maze, maze::Point { row: r, col: c });
        }
    }
}

// Maze Bound Checking

pub fn choose_arbitrary_point(maze: &maze::Maze, parity: ParityPoint) -> maze::Point {
    let init = if parity == ParityPoint::Even { 2 } else { 1 };
    for r in (init..maze.row_size() - 1).step_by(2) {
        for c in (init..maze.row_size() - 1).step_by(2) {
            if (maze[r as usize][c as usize] & maze::BUILDER_BIT) == 0 {
                return maze::Point { row: r, col: c };
            }
        }
    }
    maze::Point { row: 0, col: 0 }
}

pub fn can_build_new_square(maze: &maze::Maze, next: maze::Point) -> bool {
    return next.row > 0
        && next.row < maze.row_size() - 1
        && next.col > 0
        && next.col < maze.col_size() - 1
        && (maze[next.row as usize][next.col as usize] & maze::BUILDER_BIT) == 0;
}

pub fn has_builder_bit(maze: &maze::Maze, next: maze::Point) -> bool {
    return (maze[next.row as usize][next.col as usize] & maze::BUILDER_BIT) != 0;
}

pub fn is_square_within_perimeter_walls(maze: &maze::Maze, next: maze::Point) -> bool {
    return next.row < maze.row_size() - 1
        && next.row > 0
        && next.col < maze.col_size() - 1
        && next.col > 0;
}

// Wall Adder Helpers

pub fn build_wall_line(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::NORTH_WALL;
        maze[u_row - 1][u_col] |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::SOUTH_WALL;
        maze[u_row + 1][u_col] |= maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        wall |= maze::WEST_WALL;
        maze[u_row][u_col - 1] |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        wall |= maze::EAST_WALL;
        maze[u_row][u_col + 1] |= maze::WEST_WALL;
    }
    maze[u_row][u_col] |= wall;
    maze[u_row][u_col] |= maze::BUILDER_BIT;
    maze[u_row][u_col] &= !maze::PATH_BIT;
}

pub fn build_wall_line_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::NORTH_WALL;
        maze[u_row - 1][u_col] |= maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::SOUTH_WALL;
        maze[u_row + 1][u_col] |= maze::NORTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        wall |= maze::WEST_WALL;
        maze[u_row][u_col - 1] |= maze::EAST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        wall |= maze::EAST_WALL;
        maze[u_row][u_col + 1] |= maze::WEST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    maze[u_row][u_col] |= wall;
    maze[u_row][u_col] |= maze::BUILDER_BIT;
    maze[u_row][u_col] &= !maze::PATH_BIT;
    flush_cursor_maze_coordinate(
        maze,
        maze::Point {
            row: p.row,
            col: p.col,
        },
    );
    thread::sleep(time::Duration::from_micros(speed));
}

// Path Carving Helpers

pub fn clear_for_wall_adders(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if c == 0 || c == maze.col_size() - 1 || r == 0 || r == maze.row_size() - 1 {
                maze[r as usize][c as usize] |= maze::BUILDER_BIT;
                continue;
            }
            build_path(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn mark_origin(maze: &mut maze::Maze, walk: maze::Point, next: maze::Point) {
    let u_next_row = next.row as usize;
    let u_next_col = next.col as usize;
    if next.row > walk.row {
        maze[u_next_row][u_next_col] |= FROM_NORTH;
    } else if next.row < walk.row {
        maze[u_next_row][u_next_col] |= FROM_SOUTH;
    } else if next.col < walk.col {
        maze[u_next_row][u_next_col] |= FROM_EAST;
    } else if next.col > walk.col {
        maze[u_next_row][u_next_col] |= FROM_WEST;
    }
}

pub fn mark_origin_animated(
    maze: &mut maze::Maze,
    walk: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let u_next_row = next.row as usize;
    let u_next_col = next.col as usize;
    if next.row > walk.row {
        maze[u_next_row][u_next_col] |= FROM_NORTH;
    } else if next.row < walk.row {
        maze[u_next_row][u_next_col] |= FROM_SOUTH;
    } else if next.col < walk.col {
        maze[u_next_row][u_next_col] |= FROM_EAST;
    } else if next.col > walk.col {
        maze[u_next_row][u_next_col] |= FROM_WEST;
    }
    flush_cursor_maze_coordinate(maze, next);
    thread::sleep(time::Duration::from_micros(speed));
}

pub fn fill_maze_with_walls(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            build_wall(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn fill_maze_with_walls_animated(maze: &mut maze::Maze) {
    print::clear_screen();
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            build_wall(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn carve_path_walls(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT;
    if p.row - 1 >= 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        maze[u_row][u_col + 1] &= !maze::WEST_WALL;
    }
    maze[u_row][u_col] |= maze::BUILDER_BIT;
}

pub fn carve_path_walls_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 < maze.col_size() && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col + 1] &= !maze::WEST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col + 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    maze[u_row][u_col] |= maze::BUILDER_BIT;
}

pub fn carve_path_markings(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    let u_next_row = next.row as usize;
    let u_next_col = next.col as usize;
    let mut wall: maze::Point = cur;
    if next.row < cur.row {
        wall.row -= 1;
        maze[u_next_row][u_next_col] |= FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        maze[u_next_row][u_next_col] |= FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        maze[u_next_row][u_next_col] |= FROM_EAST;
    } else if next.col > cur.col {
        wall.col += 1;
        maze[u_next_row][u_next_col] |= FROM_WEST;
    } else {
        panic!("Wall break error, builder broke when trying step through wall.");
    }
    carve_path_walls(maze, cur);
    carve_path_walls(maze, next);
    carve_path_walls(maze, wall);
}

pub fn carve_path_markings_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: SpeedUnit,
) {
    let u_next_row = next.row as usize;
    let u_next_col = next.col as usize;
    let mut wall: maze::Point = cur;
    if next.row < cur.row {
        wall.row -= 1;
        maze[u_next_row][u_next_col] |= FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        maze[u_next_row][u_next_col] |= FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        maze[u_next_row][u_next_col] |= FROM_EAST;
    } else if next.col > cur.col {
        wall.col += 1;
        maze[u_next_row][u_next_col] |= FROM_WEST;
    } else {
        panic!("Wall break error, builder broke when trying step through wall.");
    }
    carve_path_walls_animated(maze, cur, speed);
    carve_path_walls_animated(maze, next, speed);
    carve_path_walls_animated(maze, wall, speed);
}

pub fn join_squares(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    build_path(maze, cur);
    maze[cur.row as usize][cur.col as usize] |= maze::BUILDER_BIT;
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
        panic!("Cell join error. Maze won't build");
    }
    build_path(maze, wall);
    maze[wall.row as usize][wall.col as usize] |= maze::BUILDER_BIT;
    build_path(maze, next);
    maze[next.row as usize][next.col as usize] |= maze::BUILDER_BIT;
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
        panic!("Cell join error. Maze won't build");
    }
    carve_path_walls_animated(maze, cur, speed);
    carve_path_walls_animated(maze, wall, speed);
    carve_path_walls_animated(maze, next, speed);
}

pub fn build_wall(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row - 1 >= 0 {
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        wall |= maze::SOUTH_WALL;
    }
    if p.col - 1 >= 0 {
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        wall |= maze::EAST_WALL;
    }
    maze[p.row as usize][p.col as usize] |= wall;
    maze[p.row as usize][p.col as usize] &= !maze::PATH_BIT;
}

pub fn build_wall_carefully(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::NORTH_WALL;
        maze[u_row - 1][u_col] |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::SOUTH_WALL;
        maze[u_row + 1][u_col] |= maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        wall |= maze::WEST_WALL;
        maze[u_row][u_col - 1] |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        wall |= maze::EAST_WALL;
        maze[u_row][u_col + 1] |= maze::WEST_WALL;
    }
    maze[u_row][u_col] |= wall;
    maze[u_row][u_col] &= !maze::PATH_BIT;
}

pub fn build_path(maze: &mut maze::Maze, p: maze::Point) {
    if p.row - 1 >= 0 {
        maze[(p.row - 1) as usize][p.col as usize] &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        maze[(p.row + 1) as usize][p.col as usize] &= !maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 {
        maze[p.row as usize][(p.col - 1) as usize] &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        maze[p.row as usize][(p.col + 1) as usize] &= !maze::WEST_WALL;
    }
    maze[p.row as usize][p.col as usize] |= maze::PATH_BIT;
}

pub fn build_path_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row - 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row + 1,
                col: p.col,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(
            maze,
            maze::Point {
                row: p.row,
                col: p.col - 1,
            },
        );
        thread::sleep(time::Duration::from_micros(speed));
    }
    if p.col + 1 >= 0 && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col + 1] &= !maze::EAST_WALL;
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

// Terminal Printing Helpers

pub fn flush_cursor_maze_coordinate(maze: &maze::Maze, p: maze::Point) {
    print_square(maze, p);
    print::flush();
}

pub fn print_maze_square(maze: &maze::Maze, p: maze::Point) {
    let square = &maze[p.row as usize][p.col as usize];
    print::set_cursor_position(p);
    if square & maze::PATH_BIT == 0 {
        print!("{}", maze.wall_style()[(square & maze::WALL_MASK) as usize]);
    } else if square & maze::PATH_BIT != 0 {
        print!(" ");
    } else {
        panic!("Maze square has no category");
    }
}

pub fn print_square(maze: &maze::Maze, p: maze::Point) {
    let square = &maze[p.row as usize][p.col as usize];
    print::set_cursor_position(p);
    if square & MARKERS_MASK != 0 {
        let mark = (square & MARKERS_MASK) >> MARKER_SHIFT;
        print!("{}", BACKTRACKING_SYMBOLS[mark as usize]);
    } else if square & maze::PATH_BIT == 0 {
        print!("{}", maze.wall_style()[(square & maze::WALL_MASK) as usize]);
    } else if square & maze::PATH_BIT != 0 {
        print!(" ");
    } else {
        panic!("Printed a maze square without a category.");
    }
}

pub fn clear_and_flush_grid(maze: &maze::Maze) {
    print::clear_screen();
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            print_square(maze, maze::Point { row: r, col: c });
        }
        print!("\n");
    }
    print::flush();
}

pub fn print_maze(maze: &maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            print_square(maze, maze::Point { row: r, col: c });
        }
        print!("\n");
    }
}
