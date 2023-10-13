use key;
use maze;
use print;
use print::maze_panic;

use std::{thread, time};

pub type SpeedUnit = u64;
pub type BacktrackMarker = u16;

#[derive(PartialEq, Eq)]
pub enum ParityPoint {
    Even,
    Odd,
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

// Maze Modification Helpers

pub fn build_wall_outline(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if c == 0 || c == maze.col_size() - 1 || r == 0 || r == maze.row_size() - 1 {
                maze[r as usize][c as usize] |= BUILDER_BIT;
                build_wall_carefully(maze, maze::Point { row: r, col: c });
                continue;
            }
            build_path(maze, maze::Point { row: r, col: c });
        }
    }
}

// Maze Bound Checking

pub fn choose_arbitrary_point(maze: &maze::Maze, parity: ParityPoint) -> Option<maze::Point> {
    let init = if parity == ParityPoint::Even { 2 } else { 1 };
    for r in (init..maze.row_size() - 1).step_by(2) {
        for c in (init..maze.col_size() - 1).step_by(2) {
            if (maze[r as usize][c as usize] & BUILDER_BIT) == 0 {
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
            if (maze[r as usize][c as usize] & BUILDER_BIT) == 0 {
                return Some(maze::Point { row: r, col: c });
            }
        }
    }
    None
}

pub fn can_build_new_square(maze: &maze::Maze, next: maze::Point) -> bool {
    next.row > 0
        && next.row < maze.row_size() - 1
        && next.col > 0
        && next.col < maze.col_size() - 1
        && (maze[next.row as usize][next.col as usize] & BUILDER_BIT) == 0
}

pub fn has_builder_bit(maze: &maze::Maze, next: maze::Point) -> bool {
    (maze[next.row as usize][next.col as usize] & BUILDER_BIT) != 0
}

pub fn is_square_within_perimeter_walls(maze: &maze::Maze, next: maze::Point) -> bool {
    next.row < maze.row_size() - 1 && next.row > 0 && next.col < maze.col_size() - 1 && next.col > 0
}

// Wall Adder Helpers

pub fn build_wall_line(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::NORTH_WALL;
        maze[u_row - 1][u_col] |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::SOUTH_WALL;
        maze[u_row + 1][u_col] |= maze::NORTH_WALL;
    }
    if p.col > 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        wall |= maze::WEST_WALL;
        maze[u_row][u_col - 1] |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        wall |= maze::EAST_WALL;
        maze[u_row][u_col + 1] |= maze::WEST_WALL;
    }
    maze[u_row][u_col] |= wall;
    maze[u_row][u_col] |= BUILDER_BIT;
    maze[u_row][u_col] &= !maze::PATH_BIT;
}

pub fn build_wall_line_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
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
    if p.col > 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
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
    maze[u_row][u_col] |= BUILDER_BIT;
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
    let mut wall = walk;
    if next.row > walk.row {
        wall.row += 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_NORTH;
        maze[u_next_row][u_next_col] |= FROM_NORTH;
    } else if next.row < walk.row {
        wall.row -= 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_SOUTH;
        maze[u_next_row][u_next_col] |= FROM_SOUTH;
    } else if next.col < walk.col {
        wall.col -= 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_EAST;
        maze[u_next_row][u_next_col] |= FROM_EAST;
    } else if next.col > walk.col {
        wall.col += 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_WEST;
        maze[u_next_row][u_next_col] |= FROM_WEST;
    }
    flush_cursor_maze_coordinate(maze, wall);
    thread::sleep(time::Duration::from_micros(speed));
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
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            build_wall(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn carve_path_walls(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT | BUILDER_BIT;
    if p.row > 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        maze[u_row][u_col + 1] &= !maze::WEST_WALL;
    }
}

pub fn carve_path_walls_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT | BUILDER_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row > 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
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
    if p.col > 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
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
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
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
        maze[wall.row as usize][wall.col as usize] |= FROM_SOUTH;
        maze[u_next_row][u_next_col] |= FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_NORTH;
        maze[u_next_row][u_next_col] |= FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_EAST;
        maze[u_next_row][u_next_col] |= FROM_EAST;
    } else if next.col > cur.col {
        wall.col += 1;
        maze[wall.row as usize][wall.col as usize] |= FROM_WEST;
        maze[u_next_row][u_next_col] |= FROM_WEST;
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    }
    carve_path_walls_animated(maze, cur, speed);
    carve_path_walls_animated(maze, next, speed);
    carve_path_walls_animated(maze, wall, speed);
}

pub fn join_squares(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    build_path(maze, cur);
    maze[cur.row as usize][cur.col as usize] |= BUILDER_BIT;
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
    maze[wall.row as usize][wall.col as usize] |= BUILDER_BIT;
    build_path(maze, next);
    maze[next.row as usize][next.col as usize] |= BUILDER_BIT;
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
    maze[p.row as usize][p.col as usize] |= wall;
    maze[p.row as usize][p.col as usize] &= !maze::PATH_BIT;
}

pub fn build_wall_carefully(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row > 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::NORTH_WALL;
        maze[u_row - 1][u_col] |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::SOUTH_WALL;
        maze[u_row + 1][u_col] |= maze::NORTH_WALL;
    }
    if p.col > 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
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
    if p.row > 0 {
        maze[(p.row - 1) as usize][p.col as usize] &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        maze[(p.row + 1) as usize][p.col as usize] &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
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
    if p.row > 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
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
    if p.col > 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
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
}

pub fn print_overlap_key(maze: &maze::Maze) {
    let mut key_maze = maze::Maze::new(maze::MazeArgs {
        odd_rows: THREAD_KEY_MAZE_ROWS,
        odd_cols: THREAD_KEY_MAZE_COLS,
        offset: maze::Offset {
            add_rows: maze.offset().add_rows + maze.row_size(),
            add_cols: maze.offset().add_cols,
        },
        style: WALL_STYLE_COPY_TABLE[maze.style_index()],
    });
    fill_maze_with_walls(&mut *key_maze);
    let mut cur_print = 0;
    for r in (1..key_maze.row_size() - 1).step_by(2) {
        for c in (1..key_maze.col_size() - 1).step_by((KEY_ENTRY_LEN + 1) as usize) {
            let cur_pos = maze::Point { row: r, col: c };
            for cell in c..c + KEY_ENTRY_LEN {
                build_path(&mut key_maze, maze::Point { row: r, col: cell });
            }
            print::set_cursor_position(cur_pos, key_maze.offset());
            let print_from = key::thread_color(cur_print);
            print!("{} {}", print_from.block, print_from.binary);
            cur_print += 1;
        }
    }
}

pub fn print_overlap_key_animated(maze: &maze::Maze) {
    let mut key_maze = maze::Maze::new(maze::MazeArgs {
        odd_rows: THREAD_KEY_MAZE_ROWS,
        odd_cols: THREAD_KEY_MAZE_COLS,
        offset: maze::Offset {
            add_rows: maze.offset().add_rows + maze.row_size(),
            add_cols: maze.offset().add_cols,
        },
        style: WALL_STYLE_COPY_TABLE[maze.style_index()],
    });
    fill_maze_with_walls(&mut *key_maze);
    flush_grid(&*key_maze);
    let mut cur_print = 0;
    for r in (1..key_maze.row_size() - 1).step_by(2) {
        for c in (1..key_maze.col_size() - 1).step_by((KEY_ENTRY_LEN + 1) as usize) {
            let cur_pos = maze::Point { row: r, col: c };
            for cell in c..c + KEY_ENTRY_LEN {
                build_path_animated(&mut key_maze, maze::Point { row: r, col: cell }, 250);
            }
            print::set_cursor_position(cur_pos, key_maze.offset());
            let print_from = key::thread_color(cur_print);
            print!("{} {}", print_from.block, print_from.binary);
            cur_print += 1;
        }
    }
}

// Terminal Printing Helpers

pub fn flush_cursor_maze_coordinate(maze: &maze::Maze, p: maze::Point) {
    print_square(maze, p);
    print::flush();
}

pub fn print_square(maze: &maze::Maze, p: maze::Point) {
    let square = &maze[p.row as usize][p.col as usize];
    print::set_cursor_position(p, maze.offset());
    if square & MARKERS_MASK != 0 {
        let mark = (square & MARKERS_MASK) >> MARKER_SHIFT;
        print!("{}", BACKTRACKING_SYMBOLS[mark as usize]);
    } else if square & maze::PATH_BIT == 0 {
        print!("{}", maze.wall_style()[(square & maze::WALL_MASK) as usize]);
    } else if square & maze::PATH_BIT != 0 {
        print!(" ");
    } else {
        print::maze_panic!("Printed a maze square without a category.");
    }
}

pub fn flush_grid(maze: &maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            print_square(maze, maze::Point { row: r, col: c });
        }
        println!();
    }
    print::flush();
}

const KEY_ENTRY_LEN: i32 = 8;
const THREAD_KEY_MAZE_ROWS: i32 = 2 * 2 + 1;
const THREAD_KEY_MAZE_COLS: i32 = 8 * (KEY_ENTRY_LEN + 1) + 1;
static WALL_STYLE_COPY_TABLE: [maze::MazeStyle; 6] = [
    maze::MazeStyle::Sharp,
    maze::MazeStyle::Round,
    maze::MazeStyle::Doubles,
    maze::MazeStyle::Bold,
    maze::MazeStyle::Contrast,
    maze::MazeStyle::Spikes,
];
