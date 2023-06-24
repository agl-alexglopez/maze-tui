use crate::build;
use crate::maze;

use rand::prelude::*;
use std::collections::{BinaryHeap, HashMap};

#[derive(Clone, Copy, Eq)]
struct PriorityPoint {
    priority: u8,
    p: maze::Point,
}

impl PartialEq for PriorityPoint {
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority) && self.p.eq(&other.p)
    }
}

impl PartialOrd for PriorityPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for PriorityPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

// Public Builder Function------------------------------------------------------------------------

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    let mut rng = thread_rng();
    let start = PriorityPoint {
        priority: rng.gen_range(1..=100),
        p: pick_rand_odd_point(maze),
    };
    let mut lookup_weights: HashMap<maze::Point, u8> = HashMap::from([(start.p, start.priority)]);
    let mut pq = BinaryHeap::from([start]);
    while let Some(&cur) = pq.peek() {
        let mut max_neighbor: Option<PriorityPoint> = None;
        let mut max_weight = 0;
        for dir in &build::GENERATE_DIRECTIONS {
            let next = maze::Point {
                row: cur.p.row + dir.row,
                col: cur.p.col + dir.col,
            };
            if !build::can_build_new_square(maze, next) {
                continue;
            }
            // Weights would have been randomly pre-generated anyway. Generate as we go
            // instead. However, once we choose a weight it must always be the same so
            // we cache that weight and will find it if we choose to join that square later.
            let weight = *lookup_weights.entry(next).or_insert(rng.gen_range(1..=100));
            if weight > max_weight {
                max_weight = weight;
                max_neighbor.replace(PriorityPoint {
                    priority: weight,
                    p: next,
                });
            }
        }
        match max_neighbor {
            Some(neighbor) => {
                build::join_squares(maze, cur.p, neighbor.p);
                pq.push(neighbor);
            }
            None => {
                pq.pop();
            }
        };
    }
    build::clear_and_flush_grid(maze);
}

pub fn animate_maze(maze: &mut maze::Maze, speed: build::BuilderSpeed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls_animated(maze);
    build::clear_and_flush_grid(maze);
    let mut rng = thread_rng();
    let start = PriorityPoint {
        priority: rng.gen_range(1..=100),
        p: pick_rand_odd_point(maze),
    };
    let mut lookup_weights: HashMap<maze::Point, u8> = HashMap::from([(start.p, start.priority)]);
    let mut pq = BinaryHeap::from([start]);
    while let Some(&cur) = pq.peek() {
        let mut max_neighbor: Option<PriorityPoint> = None;
        let mut max_weight = 0;
        for dir in &build::GENERATE_DIRECTIONS {
            let next = maze::Point {
                row: cur.p.row + dir.row,
                col: cur.p.col + dir.col,
            };
            if !build::can_build_new_square(maze, next) {
                continue;
            }
            let weight = *lookup_weights.entry(next).or_insert(rng.gen_range(1..=100));
            if weight > max_weight {
                max_weight = weight;
                max_neighbor.replace(PriorityPoint {
                    priority: weight,
                    p: next,
                });
            }
        }
        match max_neighbor {
            Some(neighbor) => {
                build::join_squares_animated(maze, cur.p, neighbor.p, animation);
                pq.push(neighbor);
            }
            None => {
                pq.pop();
            }
        };
    }
}

// Private Helper Function------------------------------------------------------------------------

fn pick_rand_odd_point(maze: &maze::Maze) -> maze::Point {
    let mut rand = thread_rng();
    maze::Point {
        row: 2 * rand.gen_range(1..((maze.row_size() - 2) / 2)) + 1,
        col: 2 * rand.gen_range(1..((maze.col_size() - 2) / 2)) + 1,
    }
}
