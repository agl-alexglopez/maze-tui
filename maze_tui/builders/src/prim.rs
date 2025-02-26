use crate::build;
use maze;

use rand::{
    distributions::{Distribution, Uniform},
    thread_rng, Rng,
};
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
        Some(self.cmp(other))
    }
}

impl Ord for PriorityPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
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
    let weight_range = Uniform::from(1..=100);
    let start = PriorityPoint {
        priority: weight_range.sample(&mut rng),
        p: maze::Point {
            row: 2 * rng.gen_range(1..((lk.maze.rows() - 2) / 2)) + 1,
            col: 2 * rng.gen_range(1..((lk.maze.cols() - 2) / 2)) + 1,
        },
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
            if !build::can_build_new_square(&lk.maze, next) {
                continue;
            }
            // Weights would have been randomly pre-generated anyway. Generate as we go
            // instead. However, once we choose a weight it must always be the same so
            // we cache that weight and will find it if we choose to join that square later.
            let weight = *lookup_weights
                .entry(next)
                .or_insert(weight_range.sample(&mut rng));
            if weight > max_weight {
                max_weight = weight;
                max_neighbor.replace(PriorityPoint {
                    priority: weight,
                    p: next,
                });
            }
        }
        if let Some(neighbor) = max_neighbor {
            build::join_squares(&mut lk.maze, cur.p, neighbor.p);
            pq.push(neighbor);
        } else {
            pq.pop();
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
    let weight_range = Uniform::from(1..=100);
    let start = PriorityPoint {
        priority: weight_range.sample(&mut rng),
        p: maze::Point {
            row: 2 * rng.gen_range(1..((lk.maze.rows() - 2) / 2)) + 1,
            col: 2 * rng.gen_range(1..((lk.maze.cols() - 2) / 2)) + 1,
        },
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
            if !build::can_build_new_square(&lk.maze, next) {
                continue;
            }
            // Weights would have been randomly pre-generated anyway. Generate as we go
            // instead. However, once we choose a weight it must always be the same so
            // we cache that weight and will find it if we choose to join that square later.
            let weight = *lookup_weights
                .entry(next)
                .or_insert(weight_range.sample(&mut rng));
            if weight > max_weight {
                max_weight = weight;
                max_neighbor.replace(PriorityPoint {
                    priority: weight,
                    p: next,
                });
            }
        }
        if let Some(neighbor) = max_neighbor {
            build::join_squares_history(&mut lk.maze, cur.p, neighbor.p);
            pq.push(neighbor);
        } else {
            pq.pop();
        }
    }
}
