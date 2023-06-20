use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use std::sync::{Arc, Mutex};
use std::{thread, time};

struct SolverMonitor<'a> {
    maze_monitor: Arc<Mutex<&'a mut maze::Maze>>,
    win_monitor: Arc<Mutex<Option<usize>>>,
}

impl<'a> SolverMonitor<'a> {
    fn new(maze: &'a mut maze::Maze) -> Self {
        Self {
            maze_monitor: Arc::new(Mutex::new(maze)),
            win_monitor: Arc::new(Mutex::new(None)),
        }
    }
}

struct ThreadTools {
    speed: Option<solve_util::SolveSpeedUnit>,
    starts: Vec<maze::Point>,
    paths: Vec<Vec<maze::Point>>,
}

impl ThreadTools {
    fn new() -> Self {
        Self {
            speed: None,
            starts: Vec::new(),
            paths: vec![Vec::with_capacity(solve_util::INITIAL_PATH_LEN); solve_util::NUM_THREADS],
        }
    }
}

// Public Solver Functions

pub fn solve_with_dfs_thread_hunt(maze: &mut maze::Maze) {
    let mut tools = ThreadTools::new();
    tools.starts = vec![solve_util::pick_random_point(maze); solve_util::NUM_THREADS];
    maze[tools.starts[0].row as usize][tools.starts[0].col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    let tools_arc = Arc::new(Mutex::new(tools));
    let mut monitor = SolverMonitor::new(maze);
    let mut handles = vec![];
    for i_thread in 0..solve_util::NUM_THREADS {
        let this_thread = solve_util::ThreadId {index: i_thread, paint: solve_util::THREAD_MASKS[i_thread]};
        let mut monitor_clone = SolverMonitor {maze_monitor: Arc::clone(&monitor.maze_monitor), win_monitor: Arc::clone(&monitor.win_monitor)};
        let mut tools_clone = tools_arc.clone();
        let handle = thread::spawn(move || {
            complete_hunt(&mut monitor_clone, &mut tools_clone, this_thread);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    solve_util::print_paths(maze);
    solve_util::print_overlap_key();
    solve_util::print_hunt_solution_message(*monitor.win_monitor.lock().unwrap());
    println!();
}

fn complete_hunt(monitor: &mut SolverMonitor, tools: &mut Arc<Mutex<ThreadTools>>, id: solve_util::ThreadId) {

}
