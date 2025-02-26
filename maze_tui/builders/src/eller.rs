use crate::build;
use maze;
use print;

use rand::{
    distributions::{Bernoulli, Distribution},
    thread_rng, Rng,
};
use std::collections::HashMap;

const WINDOW_SIZE: usize = 2;
const DROP_DIST: i32 = 2;
const NEIGHBOR_DIST: i32 = 2;

type SetId = usize;

struct SlidingSetWindow {
    cur_row: usize,
    width: usize,
    next_available_id: SetId,
    sets: Vec<SetId>,
}

struct IdMergeRequest {
    winner: SetId,
    loser: SetId,
}

///
/// Data only maze generator
///
pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    let mut rng = thread_rng();
    let coin = Bernoulli::new(0.66);
    let mut window = SlidingSetWindow::new(&lk.maze);
    let mut sets_in_this_row: HashMap<SetId, Vec<maze::Point>> = HashMap::new();
    for r in (1..lk.maze.rows() - 2).step_by(2) {
        window.generate_sets(window.next_row_i());
        for c in (1..lk.maze.cols() - 1).step_by(2) {
            let cur_id = window.get(window.cur_row, c as usize);
            let next = maze::Point {
                row: r,
                col: c + NEIGHBOR_DIST,
            };
            if !build::is_square_within_perimeter_walls(&lk.maze, next)
                || cur_id == window.get(window.cur_row, next.col as usize)
                || !coin.expect("Bernoulli coin flip broke").sample(&mut rng)
            {
                continue;
            }
            let neighbor_id = window.get(window.cur_row, next.col as usize);
            build::join_squares(&mut lk.maze, maze::Point { row: r, col: c }, next);
            merge_cur_row_sets(
                &mut window,
                IdMergeRequest {
                    winner: cur_id,
                    loser: neighbor_id,
                },
            );
        }

        for c in (1..lk.maze.cols() - 1).step_by(2) {
            sets_in_this_row
                .entry(window.get(window.cur_row, c as usize))
                .or_default()
                .push(maze::Point { row: r, col: c });
        }

        for set in sets_in_this_row.iter() {
            for _drop in 0..rng.gen_range(1..=set.1.len()) {
                let chose: &maze::Point = &set.1[rng.gen_range(0..set.1.len())];
                if (lk.maze.get(chose.row + DROP_DIST, chose.col) & build::BUILDER_BIT) != 0 {
                    continue;
                }
                let next_row = window.next_row_i();
                *window.get_mut(next_row, chose.col as usize) = *set.0;
                build::join_squares(
                    &mut lk.maze,
                    *chose,
                    maze::Point {
                        row: chose.row + DROP_DIST,
                        col: chose.col,
                    },
                );
            }
        }
        window.slide_window();
        sets_in_this_row.clear();
    }
    complete_final_row(&mut lk.maze, &mut window);
}

fn merge_cur_row_sets(window: &mut SlidingSetWindow, request: IdMergeRequest) {
    let row = window.cur_row;
    for id in window.row(row) {
        if *id == request.loser {
            *id = request.winner;
        }
    }
}

fn complete_final_row(maze: &mut maze::Maze, window: &mut SlidingSetWindow) {
    let r = maze.rows() - 2;
    let set_r = window.cur_row;
    for c in (1..maze.cols() - 2).step_by(2) {
        let this_id = window.get(set_r, c as usize);
        let next = maze::Point {
            row: r,
            col: c + NEIGHBOR_DIST,
        };
        let neighbor_id = window.get(set_r, (c + NEIGHBOR_DIST) as usize);
        if this_id == neighbor_id {
            continue;
        }
        build::join_squares(maze, maze::Point { row: r, col: c }, next);
        for set_elem in (next.col..maze.cols() - 1).step_by(2) {
            if window.get(set_r, set_elem as usize) == neighbor_id {
                *window.get_mut(set_r, set_elem as usize) = this_id;
            }
        }
    }
}

///
/// History based generator for animation and playback.
///
pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_history_with_walls(&mut lk.maze);
    let mut rng = thread_rng();
    let coin = Bernoulli::new(0.66);
    let mut window = SlidingSetWindow::new(&lk.maze);
    let mut sets_in_this_row: HashMap<SetId, Vec<maze::Point>> = HashMap::new();
    for r in (1..lk.maze.rows() - 2).step_by(2) {
        window.generate_sets(window.next_row_i());
        for c in (1..lk.maze.cols() - 1).step_by(2) {
            let cur_id = window.get(window.cur_row, c as usize);
            let next = maze::Point {
                row: r,
                col: c + NEIGHBOR_DIST,
            };
            if !build::is_square_within_perimeter_walls(&lk.maze, next)
                || cur_id == window.get(window.cur_row, next.col as usize)
                || !coin.expect("Bernoulli coin flip broke").sample(&mut rng)
            {
                continue;
            }
            let neighbor_id = window.get(window.cur_row, next.col as usize);
            build::join_squares_history(&mut lk.maze, maze::Point { row: r, col: c }, next);
            merge_cur_row_sets(
                &mut window,
                IdMergeRequest {
                    winner: cur_id,
                    loser: neighbor_id,
                },
            );
        }

        for c in (1..lk.maze.cols() - 1).step_by(2) {
            sets_in_this_row
                .entry(window.get(window.cur_row, c as usize))
                .or_default()
                .push(maze::Point { row: r, col: c });
        }

        for set in sets_in_this_row.iter() {
            for _drop in 0..rng.gen_range(1..=set.1.len()) {
                let chose: &maze::Point = &set.1[rng.gen_range(0..set.1.len())];
                if build::is_built(lk.maze.get(chose.row + DROP_DIST, chose.col)) {
                    continue;
                }
                let next_row = window.next_row_i();
                *window.get_mut(next_row, chose.col as usize) = *set.0;
                build::join_squares_history(
                    &mut lk.maze,
                    *chose,
                    maze::Point {
                        row: chose.row + DROP_DIST,
                        col: chose.col,
                    },
                );
            }
        }
        window.slide_window();
        sets_in_this_row.clear();
    }
    complete_final_row_history(&mut lk.maze, &mut window);
}

fn complete_final_row_history(maze: &mut maze::Maze, window: &mut SlidingSetWindow) {
    let r = maze.rows() - 2;
    let set_r = window.cur_row;
    for c in (1..maze.cols() - 2).step_by(2) {
        let this_id = window.get(set_r, c as usize);
        let next = maze::Point {
            row: r,
            col: c + NEIGHBOR_DIST,
        };
        let neighbor_id = window.get(set_r, (c + NEIGHBOR_DIST) as usize);
        if this_id == neighbor_id {
            continue;
        }
        build::join_squares_history(maze, maze::Point { row: r, col: c }, next);
        for set_elem in (next.col..maze.cols() - 1).step_by(2) {
            if window.get(set_r, set_elem as usize) == neighbor_id {
                *window.get_mut(set_r, set_elem as usize) = this_id;
            }
        }
    }
}

///
/// Helper data structure implementation for managing sets available to all.
///
impl SlidingSetWindow {
    fn new(maze: &maze::Maze) -> Self {
        let mut ids = vec![0; WINDOW_SIZE * maze.cols() as usize];
        for (i, v) in ids.iter_mut().enumerate() {
            *v = i;
        }
        Self {
            cur_row: 0,
            width: maze.cols() as usize,
            next_available_id: maze.cols() as usize,
            sets: ids,
        }
    }

    fn slide_window(&mut self) {
        self.cur_row = (self.cur_row + 1) % 2;
    }

    fn next_row_i(&self) -> usize {
        (self.cur_row + 1) % WINDOW_SIZE
    }

    fn get(&self, row: usize, col: usize) -> SetId {
        self.sets[row * self.width + col]
    }

    fn get_mut(&mut self, row: usize, col: usize) -> &mut SetId {
        &mut self.sets[row * self.width + col]
    }

    fn row(&mut self, row: usize) -> &mut [SetId] {
        &mut self.sets[(row * self.width)..(row * self.width + self.width)]
    }

    fn generate_sets(&mut self, row: usize) {
        if row >= WINDOW_SIZE {
            print::maze_panic!(
                "Cannot generate sets for a row that does not exist, row: {}",
                row
            );
        }
        for e in &mut self.sets[(row * self.width)..(row * self.width + self.width)].iter_mut() {
            *e = self.next_available_id;
            self.next_available_id += 1;
        }
    }
}
