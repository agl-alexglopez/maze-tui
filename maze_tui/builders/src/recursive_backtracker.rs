use crate::build;
use maze;
use rand::{seq::SliceRandom, thread_rng, Rng};

///
/// Data only maze generator
///
pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.rows() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.cols() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    'descending: loop {
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
                continue 'descending;
            }
        }
        let dir: build::BacktrackMarker = lk.maze.get(cur.row, cur.col) & build::MARKERS_MASK;
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

///
/// History based generator for animation and playback.
///
pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("builder could not take lock"),
    };
    build::fill_maze_history_with_walls(&mut lk.maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.rows() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.cols() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; build::NUM_DIRECTIONS] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    'descending: loop {
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
                continue 'descending;
            }
        }
        let dir: build::BacktrackMarker = lk.maze.get(cur.row, cur.col) & build::MARKERS_MASK;
        // The solvers will need these bits later so we need to clear bits.
        let half: &maze::Point = &build::BACKTRACKING_HALF_POINTS[dir as usize];
        let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
        let half_step = maze::Point {
            row: cur.row + half.row,
            col: cur.col + half.col,
        };
        let square = lk.maze.get(cur.row, cur.col);
        let half_step_square = lk.maze.get(half_step.row, half_step.col);
        lk.maze.build_history.push(maze::Delta {
            id: cur,
            before: square,
            after: square & !build::MARKERS_MASK,
            burst: 1,
        });
        lk.maze.build_history.push(maze::Delta {
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
