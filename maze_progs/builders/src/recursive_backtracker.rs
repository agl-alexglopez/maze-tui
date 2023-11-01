use crate::build;
use maze;
use rand::{seq::SliceRandom, thread_rng, Rng};
use speed;
use std::{thread, time};

// Backtracking was too fast because it just clears square. Slow down for animation.
const BACKTRACK_DELAY: build::SpeedUnit = 8;

pub fn generate_history(monitor: monitor::SolverMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("builder could not take lock"),
    };
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    'branching: loop {
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                build::carve_path_history(&mut lk.maze, cur, branch);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker = (lk.maze[cur.row as usize][cur.col as usize]
            & build::MARKERS_MASK)
            >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        let square = lk.maze[cur.row as usize][cur.col as usize];
        let half_step_square = lk.maze[half_step.row as usize][half_step.col as usize];
        lk.maze.build_history.push(maze::Delta {
            p: half_step,
            before: half_step_square,
            after: half_step_square & !build::MARKERS_MASK,
            burst: 1,
        });
        lk.maze.build_history.push(maze::Delta {
            p: cur,
            before: square,
            after: square & !build::MARKERS_MASK,
            burst: 1,
        });
        lk.maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
        lk.maze[half_step.row as usize][half_step.col as usize] &= !build::MARKERS_MASK;
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
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
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
        maze[half_step.row as usize][half_step.col as usize] &= !build::MARKERS_MASK;
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

pub fn animate_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    if maze.is_mini() {
        animate_mini_maze(maze, speed);
        return;
    }
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(maze);
    build::flush_grid(maze);
    build::print_overlap_key_animated(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
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

fn animate_mini_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(maze);
    build::flush_grid(maze);
    build::print_overlap_key_animated(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
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
                build::carve_mini_markings_animated(maze, cur, branch, animation);
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
        build::flush_mini_backtracker_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        build::flush_mini_backtracker_coordinate(maze, cur);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}
