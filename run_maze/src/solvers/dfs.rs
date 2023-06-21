use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use std::sync::{Arc, Mutex};
use std::{thread, time};

struct SolverMonitor {
    maze: Box<maze::Maze>,
    win: Option<usize>,
}

impl SolverMonitor {
    fn new(static_maze: Box<maze::Maze>) -> Self {
        Self {
            maze: static_maze,
            win: None,
        }
    }
}

// Public Solver Functions

pub fn animate_with_dfs_thread_hunt(mut maze: Box<maze::Maze>, speed: solve_util::SolverSpeed) {
    print_util::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve_util::print_overlap_key();
    let start: maze::Point = solve_util::pick_random_point(&maze);
    maze[start.row as usize][start.col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    let row_size = maze.row_size();
    let animation = solve_util::SOLVER_SPEEDS[speed as usize];
    solve_util::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    let monitor = Arc::new(Mutex::new(SolverMonitor::new(maze)));
    let mut handles = vec![];
    for i_thread in 0..solve_util::NUM_THREADS {
        let this_thread = solve_util::ThreadId {
            index: i_thread,
            paint: solve_util::THREAD_MASKS[i_thread],
        };
        let mut monitor_clone = monitor.clone();
        let handle = thread::spawn(move || {
            animate_hunt(&mut monitor_clone, this_thread, start, animation);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    print_util::set_cursor_position(maze::Point {
        row: row_size + solve_util::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
        col: 0,
    });
    solve_util::print_hunt_solution_message(monitor.lock().unwrap().win);
    println!();
}

fn animate_hunt(
    monitor: &mut Arc<Mutex<SolverMonitor>>,
    id: solve_util::ThreadId,
    start: maze::Point,
    speed: solve_util::SolveSpeedUnit,
) {
    let seen: solve_util::ThreadCache = id.paint << solve_util::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve_util::INITIAL_PATH_LEN);
    dfs.push(start);
    while !dfs.is_empty() {
        let cur: maze::Point = dfs[dfs.len() - 1];

        // A closure helps because it creates a scope for the lock. Otherwise I would have to cover
        // all the cases where I need to drop the lock explicitly, especially if there is no
        // winner and we continue with function logic.
        let mut is_winner = || -> bool {
            let mut win_lock = monitor.lock().unwrap();
            if win_lock.win.is_some() {
                return true;
            }
            if (win_lock.maze[cur.row as usize][cur.col as usize] & solve_util::FINISH_BIT) != 0 {
                if win_lock.win.is_none() {
                    let _ = win_lock.win.insert(id.index);
                }
                let _ = dfs.pop();
                return true;
            }
            win_lock.maze[cur.row as usize][cur.col as usize] |= seen;
            win_lock.maze[cur.row as usize][cur.col as usize] |= id.paint;
            solve_util::flush_cursor_path_coordinate(&win_lock.maze, cur);
            return false;
        };
        if is_winner() {
            return;
        }

        thread::sleep(time::Duration::from_micros(speed));

        // Bias threads towards their original dispatch direction. This is do-while loop.
        let mut i = id.index;
        let mut found_branch = false;
        'search: while {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            let bool_lock = monitor.lock().unwrap(); // LOCK
            let push_next: bool =
                (bool_lock.maze[next.row as usize][next.col as usize] & seen) == 0
                    && (bool_lock.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0;

            if push_next {
                found_branch = true;
                dfs.push(next);
                break 'search;
            }
            i = (i + 1) % solve_util::NUM_DIRECTIONS;
            i != id.index
        } {}

        if !found_branch {
            let mut paint_lock = monitor.lock().unwrap();  // LOCK
            paint_lock.maze[cur.row as usize][cur.col as usize] &= !id.paint;
            solve_util::flush_cursor_path_coordinate(&paint_lock.maze, cur);
            thread::sleep(time::Duration::from_micros(speed));
            let _ = dfs.pop();
        }
    }
}
