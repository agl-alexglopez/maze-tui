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
    build::fill_maze_history_with_walls(&mut lk.maze);
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
        let dir: build::BacktrackMarker =
            (lk.maze.get(cur.row, cur.col) & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        let square = lk.maze.get(cur.row, cur.col);
        let half_step_square = lk.maze.get(half_step.row, half_step.col);
        lk.maze.build_history.push(tape::Delta {
            id: cur,
            before: square,
            after: square & !build::MARKERS_MASK,
            burst: 1,
        });
        lk.maze.build_history.push(tape::Delta {
            id: half_step,
            before: half_step_square,
            after: half_step_square & !build::MARKERS_MASK,
            burst: 1,
        });
        *lk.maze.get_mut(cur.row, cur.col) &= !build::MARKERS_MASK;
        *lk.maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

pub fn generate_maze(monitor: monitor::SolverReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
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
                build::carve_path_markings(&mut lk.maze, cur, branch);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker =
            (lk.maze.get(cur.row, cur.col) & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        *lk.maze.get_mut(cur.row, cur.col) &= !build::MARKERS_MASK;
        *lk.maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

pub fn animate_maze(monitor: monitor::SolverReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    if lk.maze.is_mini() {
        drop(lk);
        animate_mini_maze(monitor, speed);
        return;
    }
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
    build::print_overlap_key_animated(&lk.maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    'branching: loop {
        if monitor.exit() {
            return;
        }
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                build::carve_path_markings_animated(&mut lk.maze, cur, branch, animation);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker =
            (lk.maze.get(cur.row, cur.col) & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        *lk.maze.get_mut(cur.row, cur.col) &= !build::MARKERS_MASK;
        *lk.maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        build::flush_cursor_maze_coordinate(&lk.maze, half_step);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        build::flush_cursor_maze_coordinate(&lk.maze, cur);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}

fn animate_mini_maze(monitor: monitor::SolverReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
    build::print_overlap_key_animated(&lk.maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    'branching: loop {
        if monitor.exit() {
            return;
        }
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                build::carve_mini_markings_animated(&mut lk.maze, cur, branch, animation);
                cur = branch;
                continue 'branching;
            }
        }
        let dir: build::BacktrackMarker =
            (lk.maze.get(cur.row, cur.col) & build::MARKERS_MASK) >> build::MARKER_SHIFT;
        // The solvers will need these bits later so we need to clear bits.
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        *lk.maze.get_mut(cur.row, cur.col) &= !build::MARKERS_MASK;
        *lk.maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        build::flush_mini_backtracker_coordinate(&lk.maze, half_step);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        build::flush_mini_backtracker_coordinate(&lk.maze, cur);
        thread::sleep(time::Duration::from_micros(animation * BACKTRACK_DELAY));
        cur.row += backtracking.row;
        cur.col += backtracking.col;

        if cur == start {
            return;
        }
    }
}
