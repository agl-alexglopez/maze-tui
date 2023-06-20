use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time};

use std::cell::RefCell;

struct SolverMonitor<'a> {
    maze_monitor: Arc<Mutex<RefCell<&'a mut maze::Maze>>>,
    win_monitor: Arc<Mutex<Option<usize>>>,
}

impl<'a> SolverMonitor<'a> {
    fn new(maze: &'a mut maze::Maze) -> Self {
        Self {
            maze_monitor: Arc::new(Mutex::new(RefCell::new(maze))),
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

pub fn animate_with_dfs_thread_hunt(maze: &'static mut maze::Maze, speed: solve_util::SolverSpeed) {
    print_util::set_cursor_position( maze::Point{row: maze.row_size(), col: 0});
    solve_util::print_overlap_key();
    let mut tools =  ThreadTools::new();
    let animation = tools.speed.insert(solve_util::SOLVER_SPEEDS[speed as usize]);
    tools.starts = vec![solve_util::pick_random_point(maze); solve_util::NUM_THREADS];
    maze[tools.starts[0].row as usize][tools.starts[0].col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    let row_size = maze.row_size();
    solve_util::flush_cursor_path_coordinate(maze, finish);
    thread::sleep(time::Duration::from_micros(*animation));

    let tools_lock = RwLock::new(tools);
    let monitor = SolverMonitor::new(maze);
    let mut handles = vec![];
    for i_thread in 0..solve_util::NUM_THREADS {
        let this_thread = solve_util::ThreadId {index: i_thread, paint: solve_util::THREAD_MASKS[i_thread]};
        let mut monitor_clone = SolverMonitor {maze_monitor: Arc::clone(&monitor.maze_monitor), win_monitor: Arc::clone(&monitor.win_monitor)};
        let handle = thread::spawn(move || {
            animate_hunt(&mut monitor_clone, tools_lock, this_thread);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    print_util::set_cursor_position(maze::Point {row: row_size + solve_util::OVERLAP_KEY_AND_MESSAGE_HEIGHT, col: 0});
    solve_util::print_hunt_solution_message(*monitor.win_monitor.lock().unwrap());
    println!();
}

fn animate_hunt(monitor: &mut SolverMonitor, tools: RwLock<ThreadTools>, id: solve_util::ThreadId) {
}
