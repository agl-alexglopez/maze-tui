use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use std::{thread, time};
use std::sync::{Arc, Mutex};

struct SolverMonitor<'a> {
    maze_ref: &'a mut maze::Maze,
    winning_index: Option<usize>
}

impl<'a> SolverMonitor<'a> {
    fn new(maze: &'a mut maze::Maze) -> Self {
        Self {maze_ref: maze, winning_index: None}
    }
}

struct SolverPack<'a> {
    monitor: &'a mut Arc<Mutex<SolverMonitor<'a>>>,
    speed: Option<solve_util::SolverSpeed>,
    starts: Vec<maze::Point>,
    thread_paths: Vec<maze::Point>
}

