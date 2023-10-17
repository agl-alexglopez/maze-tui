use crate::build;
use crate::disjoint;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::collections::HashMap;

// Public Builder Functions-----------------------------------------------------------------------

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    let walls = load_shuffled_walls(maze);
    let ids = tag_cells(maze);
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
                    build::join_squares(maze, above, below);
                }
            } else {
                print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
            }
            continue;
        }
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
                build::join_squares(maze, right, left);
            }
        } else {
            print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
        }
    }
}

pub fn animate_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(maze);
    build::flush_grid(maze);
    build::print_overlap_key_animated(maze);
    let walls = load_shuffled_walls(maze);
    let ids = tag_cells(maze);
    let mut sets = disjoint::DisjointSet::new(ids.len());

    for w in &walls {
        if maze.exit() {
            return;
        }
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
                    build::join_squares_animated(maze, above, below, animation);
                }
                continue;
            }
            print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
        }
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
                build::join_squares_animated(maze, left, right, animation);
            }
            continue;
        }
        print::maze_panic!("Kruskal couldn't find a cell id. Build broke.");
    }
}

// Private Helper Functions-----------------------------------------------------------------------

fn load_shuffled_walls(maze: &maze::Maze) -> Vec<maze::Point> {
    let mut walls = Vec::new();
    for r in (1..maze.row_size() - 1).step_by(2) {
        for c in (2..maze.col_size() - 1).step_by(2) {
            walls.push(maze::Point { row: r, col: c });
        }
    }
    for r in (2..maze.row_size() - 1).step_by(2) {
        for c in (1..maze.col_size() - 1).step_by(2) {
            walls.push(maze::Point { row: r, col: c });
        }
    }
    walls.shuffle(&mut thread_rng());
    walls
}

fn tag_cells(maze: &maze::Maze) -> HashMap<maze::Point, usize> {
    let mut set_ids = HashMap::new();
    let mut id = 0;
    for r in (1..maze.row_size() - 1).step_by(2) {
        for c in (1..maze.col_size() - 1).step_by(2) {
            set_ids.insert(maze::Point { row: r, col: c }, id);
            id += 1;
        }
    }
    set_ids
}
