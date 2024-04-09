use crate::build;
use crate::disjoint;
use maze;
use print;

use rand::prelude::*;
use std::collections::HashMap;

///
/// Data only maze generator
///

pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    let walls = load_shuffled_walls(&lk.maze);
    let ids = tag_cells(&lk.maze);
    let mut sets = disjoint::DisjointSet::new(ids.len());

    for w in &walls {
        if w.row % 2 == 0 {
            let above = maze::Point {
                row: w.row - 1,
                col: w.col,
            };
            let below = maze::Point {
                row: w.row + 1,
                col: w.col,
            };
            if let (Some(a_id), Some(b_id)) = (ids.get(&above), ids.get(&below)) {
                if sets.made_union(*a_id, *b_id) {
                    build::join_squares(&mut lk.maze, above, below);
                }
            } else {
                print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
            }
        } else {
            let left = maze::Point {
                row: w.row,
                col: w.col - 1,
            };
            let right = maze::Point {
                row: w.row,
                col: w.col + 1,
            };
            if let (Some(l_id), Some(r_id)) = (ids.get(&left), ids.get(&right)) {
                if sets.made_union(*l_id, *r_id) {
                    build::join_squares(&mut lk.maze, right, left);
                }
            } else {
                print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
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
    let walls = load_shuffled_walls(&lk.maze);
    let ids = tag_cells(&lk.maze);
    let mut sets = disjoint::DisjointSet::new(ids.len());

    for w in &walls {
        if w.row % 2 == 0 {
            let above = maze::Point {
                row: w.row - 1,
                col: w.col,
            };
            let below = maze::Point {
                row: w.row + 1,
                col: w.col,
            };
            if let (Some(a_id), Some(b_id)) = (ids.get(&above), ids.get(&below)) {
                if sets.made_union(*a_id, *b_id) {
                    build::join_squares_history(&mut lk.maze, above, below);
                }
            } else {
                print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
            }
        } else {
            let left = maze::Point {
                row: w.row,
                col: w.col - 1,
            };
            let right = maze::Point {
                row: w.row,
                col: w.col + 1,
            };
            if let (Some(l_id), Some(r_id)) = (ids.get(&left), ids.get(&right)) {
                if sets.made_union(*l_id, *r_id) {
                    build::join_squares_history(&mut lk.maze, right, left);
                }
            } else {
                print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
            }
        }
    }
}

///
/// Data only helpers available to all.
///

fn load_shuffled_walls(maze: &maze::Maze) -> Vec<maze::Point> {
    let mut walls = Vec::new();
    for r in (1..maze.rows() - 1).step_by(2) {
        for c in (2..maze.cols() - 1).step_by(2) {
            walls.push(maze::Point { row: r, col: c });
        }
    }
    for r in (2..maze.rows() - 1).step_by(2) {
        for c in (1..maze.cols() - 1).step_by(2) {
            walls.push(maze::Point { row: r, col: c });
        }
    }
    walls.shuffle(&mut thread_rng());
    walls
}

fn tag_cells(maze: &maze::Maze) -> HashMap<maze::Point, usize> {
    let mut set_ids = HashMap::new();
    let mut id = 0;
    for r in (1..maze.rows() - 1).step_by(2) {
        for c in (1..maze.cols() - 1).step_by(2) {
            set_ids.insert(maze::Point { row: r, col: c }, id);
            id += 1;
        }
    }
    set_ids
}
