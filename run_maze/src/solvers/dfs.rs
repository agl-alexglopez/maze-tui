use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use std::{thread, time};

// Public Solver Functions

pub fn solve_with_dfs_thread_hunt(mut maze: maze::BoxMaze) {}

pub fn animate_with_dfs_thread_hunt(mut maze: maze::BoxMaze, speed: solve_util::SolverSpeed) {
    print_util::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve_util::print_overlap_key();
    let all_start: maze::Point = solve_util::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    let row_size = maze.row_size();
    let animation = solve_util::SOLVER_SPEEDS[speed as usize];
    solve_util::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    let monitor = solve_util::Solver::new(maze);
    let mut handles = vec![];
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        let handle = thread::spawn(move || {
            animate_hunt(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_start,
                    speed: animation,
                },
            );
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

fn animate_hunt(monitor: &mut solve_util::SolverMonitor, guide: solve_util::ThreadGuide) {
    let seen: solve_util::ThreadCache = guide.paint << solve_util::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve_util::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    while !dfs.is_empty() {
        let cur: maze::Point = dfs[dfs.len() - 1];

        let mut win_lock = monitor.lock().unwrap();
        if win_lock.win.is_some() {
            return;
        }
        if (win_lock.maze[cur.row as usize][cur.col as usize] & solve_util::FINISH_BIT) != 0 {
            if win_lock.win.is_none() {
                let _ = win_lock.win.insert(guide.index);
            }
            let _ = dfs.pop();
            return;
        }
        win_lock.maze[cur.row as usize][cur.col as usize] |= seen;
        win_lock.maze[cur.row as usize][cur.col as usize] |= guide.paint;
        solve_util::flush_cursor_path_coordinate(&win_lock.maze, cur);
        drop(win_lock);

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        let mut found_branch = false;
        'search: while {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            // A manual drop here because threads can then manage their own data structures.
            let bool_lock = monitor.lock().unwrap();
            let push_next: bool = (bool_lock.maze[next.row as usize][next.col as usize] & seen)
                == 0
                && (bool_lock.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0;
            drop(bool_lock);

            if push_next {
                found_branch = true;
                dfs.push(next);
                break 'search;
            }
            i = (i + 1) % solve_util::NUM_DIRECTIONS;
            i != guide.index
        } {}

        if !found_branch {
            let mut paint_lock = monitor.lock().unwrap();
            paint_lock.maze[cur.row as usize][cur.col as usize] &= !guide.paint;
            solve_util::flush_cursor_path_coordinate(&paint_lock.maze, cur);
            drop(paint_lock);
            thread::sleep(time::Duration::from_micros(guide.speed));
            dfs.pop();
        }
    }
}
