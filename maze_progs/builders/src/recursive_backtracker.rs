use crate::build;
use maze;
use speed;

use crossterm::{
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor},
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{self, io, thread, time};

// Backtracking was too fast because it just clears square. Slow down for animation.
const BACKTRACK_DELAY: build::SpeedUnit = 8;

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    let mut cur: maze::Point = start;
    'branching: loop {
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(maze, branch) {
                build::carve_path_markings(maze, cur, branch);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker =
            (maze[cur.row as usize][cur.col as usize] & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

pub fn animate_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(maze);
    build::flush_grid(maze);
    build::print_overlap_key_animated(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    let mut cur: maze::Point = start;
    'branching: loop {
        if maze.exit() {
            return;
        }
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(maze, branch) {
                build::carve_path_markings_animated(maze, cur, branch, animation);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker =
            (maze[cur.row as usize][cur.col as usize] & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
        maze[half_step.row as usize][half_step.col as usize] &= !build::MARKERS_MASK;
        build::flush_cursor_maze_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        build::flush_cursor_maze_coordinate(maze, cur);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

pub fn animate_mini_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(maze);
    flush_grid(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    let mut cur: maze::Point = start;
    'branching: loop {
        if maze.exit() {
            return;
        }
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(maze, branch) {
                carve_path_markings_animated(maze, cur, branch, animation);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker =
            (maze[cur.row as usize][cur.col as usize] & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
        maze[half_step.row as usize][half_step.col as usize] &= !build::MARKERS_MASK;
        print::set_cursor_position(
            maze::Point {
                row: half_step.row / 2,
                col: half_step.col,
            },
            maze.offset(),
        );
        flush_cursor_maze_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        print::set_cursor_position(
            maze::Point {
                row: cur.row / 2,
                col: cur.col,
            },
            maze.offset(),
        );
        flush_cursor_maze_coordinate(maze, cur);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

fn flush_grid(maze: &maze::Maze) {
    for r in 0..(maze.row_size() + 2 - 1) / 2 {
        for c in 0..maze.col_size() {
            if r == ((maze.row_size() + 2 - 1) / 2) - 1 {
                print::set_cursor_position(maze::Point { row: r, col: c }, maze.offset());
                queue!(io::stdout(), Print('▀'),).expect("Could not print wall.");
            } else {
                print::set_cursor_position(maze::Point { row: r, col: c }, maze.offset());
                queue!(io::stdout(), Print('█'),).expect("Could not print wall.");
            }
        }
        match queue!(io::stdout(), Print('\n')) {
            Ok(_) => {}
            Err(_) => print::maze_panic!("Couldn't print square."),
        };
    }
    print::flush();
}

fn carve_path_markings_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: build::SpeedUnit,
) {
    let u_next_row = next.row as usize;
    let u_next_col = next.col as usize;
    let mut wall: maze::Point = cur;
    if next.row < cur.row {
        wall.row -= 1;
        maze[wall.row as usize][wall.col as usize] |= build::FROM_SOUTH;
        maze[u_next_row][u_next_col] |= build::FROM_SOUTH;
    } else if next.row > cur.row {
        wall.row += 1;
        maze[wall.row as usize][wall.col as usize] |= build::FROM_NORTH;
        maze[u_next_row][u_next_col] |= build::FROM_NORTH;
    } else if next.col < cur.col {
        wall.col -= 1;
        maze[wall.row as usize][wall.col as usize] |= build::FROM_EAST;
        maze[u_next_row][u_next_col] |= build::FROM_EAST;
    } else if next.col > cur.col {
        wall.col += 1;
        maze[wall.row as usize][wall.col as usize] |= build::FROM_WEST;
        maze[u_next_row][u_next_col] |= build::FROM_WEST;
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    }
    carve_path_walls_animated(maze, cur, speed);
    carve_path_walls_animated(maze, next, speed);
    carve_path_walls_animated(maze, wall, speed);
}

fn carve_path_walls_animated(maze: &mut maze::Maze, p: maze::Point, speed: build::SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT | build::BUILDER_BIT;
    print::set_cursor_position(
        maze::Point {
            row: (p.row) / 2,
            col: p.col,
        },
        maze.offset(),
    );
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_micros(speed));
    if p.row > 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
        print::set_cursor_position(
            maze::Point {
                row: (p.row - 1) / 2,
                col: p.col,
            },
            maze.offset(),
        );
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
        print::set_cursor_position(
            maze::Point {
                row: (p.row + 1) / 2,
                col: p.col,
            },
            maze.offset(),
        );
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
        print::set_cursor_position(
            maze::Point {
                row: (p.row) / 2,
                col: p.col - 1,
            },
            maze.offset(),
        );
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
        print::set_cursor_position(
            maze::Point {
                row: (p.row) / 2,
                col: p.col + 1,
            },
            maze.offset(),
        );
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

fn flush_cursor_maze_coordinate(maze: &maze::Maze, p: maze::Point) {
    let mut square = maze[p.row as usize][p.col as usize];
    if square & maze::PATH_BIT == 0 {
        if p.row % 2 != 0 {
            if p.row - 1 < 0 {
                return;
            }
            if (maze[(p.row - 1) as usize][p.col as usize] & maze::PATH_BIT) == 0 {
                execute!(io::stdout(), Print('█'), ResetColor).expect("Could not print.");
            } else {
                square = maze[(p.row - 1) as usize][p.col as usize];
                let mark = build::BACKTRACKING_SYMBOLS
                    [((square & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize];
                execute!(
                    io::stdout(),
                    SetBackgroundColor(Color::AnsiValue(mark.ansi)),
                    Print('▄'),
                    ResetColor
                )
                .expect("Could not print.");
            }
        } else {
            if p.row + 1 >= maze.row_size() {
                return;
            }
            if (maze[(p.row + 1) as usize][p.col as usize] & maze::PATH_BIT) == 0 {
                execute!(io::stdout(), Print('█'), ResetColor).expect("Could not print.");
            } else {
                square = maze[(p.row + 1) as usize][p.col as usize];
                let mark = build::BACKTRACKING_SYMBOLS
                    [((square & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize];
                execute!(
                    io::stdout(),
                    SetBackgroundColor(Color::AnsiValue(mark.ansi)),
                    Print('▀'),
                    ResetColor
                )
                .expect("Could not print.");
            }
        }
    } else if square & maze::PATH_BIT != 0 {
        if p.row % 2 != 0 {
            if p.row - 1 < 0 {
                return;
            }
            if (maze[(p.row - 1) as usize][p.col as usize] & maze::PATH_BIT) != 0 {
                let mark = build::BACKTRACKING_SYMBOLS
                    [((square & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize];
                execute!(
                    io::stdout(),
                    SetBackgroundColor(Color::AnsiValue(mark.ansi)),
                    Print(' '),
                    ResetColor
                )
                .expect("Could not print.");
            } else {
                let mark = build::BACKTRACKING_SYMBOLS
                    [((square & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize];
                execute!(
                    io::stdout(),
                    SetBackgroundColor(Color::AnsiValue(mark.ansi)),
                    Print('▀'),
                    ResetColor
                )
                .expect("Could not print.");
            }
        } else {
            if p.row + 1 >= maze.row_size() {
                return;
            }
            if (maze[(p.row + 1) as usize][p.col as usize] & maze::PATH_BIT) != 0 {
                let mark = build::BACKTRACKING_SYMBOLS
                    [((square & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize];
                execute!(
                    io::stdout(),
                    SetBackgroundColor(Color::AnsiValue(mark.ansi)),
                    Print(' '),
                    ResetColor
                )
                .expect("Could not print.");
            } else {
                let mark = build::BACKTRACKING_SYMBOLS
                    [((square & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize];
                execute!(
                    io::stdout(),
                    SetBackgroundColor(Color::AnsiValue(mark.ansi)),
                    Print(' '),
                    ResetColor
                )
                .expect("Could not print.");
            }
        }
    } else {
        print::maze_panic!("Printed a maze square without a category.");
    }
}
