use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use std::{thread, time};
use std::sync::{Arc, Mutex};
use rand::prelude::*;
use std::collections::{VecDeque, HashMap, HashSet};

// Public Solver Functions-------------------------------------------------------------------------

pub fn solve_with_bfs_thread_hunt(mut maze: maze::BoxMaze) {
    let all_start: maze::Point = solve_util::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;

    let paths: Arc<Mutex<Vec<Vec<maze::Point>>>> = Arc::new(Mutex::new(vec![Vec::with_capacity(solve_util::INITIAL_PATH_LEN);4]));
    let monitor = solve_util::Solver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        let paths_clone = paths.clone();
        handles.push(thread::spawn(move || {
            complete_hunt(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_start,
                    speed: 0,
                },
                paths_clone,
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut maze_lock = monitor.lock().unwrap();
    let path_lock = paths.lock().unwrap();

    if maze_lock.win.is_some() {
        let color = solve_util::THREAD_MASKS[maze_lock.win.unwrap()];
        for p in &path_lock[maze_lock.win.unwrap()] {
            maze_lock.maze[p.row as usize][p.col as usize] &= !solve_util::THREAD_MASK;
            maze_lock.maze[p.row as usize][p.col as usize] |= color;
        }
    }

    solve_util::print_paths(&monitor.lock().unwrap().maze);
    solve_util::print_overlap_key();
    solve_util::print_hunt_solution_message(monitor.lock().unwrap().win);
    println!();
}

pub fn animate_with_bfs_thread_hunt(mut maze: maze::BoxMaze, speed: solve_util::SolverSpeed) {}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn complete_hunt(monitor: &mut solve_util::SolverMonitor, guide: solve_util::ThreadGuide, paths: Arc<Mutex<Vec<Vec<maze::Point>>>>) {
    let mut seen: HashMap<maze::Point, maze::Point> = HashMap::new();
    seen.insert(guide.start, maze::Point {row: -1, col: -1});
    let mut bfs: VecDeque<maze::Point> = VecDeque::new();
    bfs.push_back(guide.start);

    let mut cur = guide.start;
    while !bfs.is_empty() {
        let win_lock = monitor.lock().unwrap();
        if win_lock.win.is_some() {
            drop(win_lock);
            break;
        }
        drop(win_lock);
        cur = bfs.pop_front().unwrap();

        let mut finish_lock = monitor.lock().unwrap();
        if (finish_lock.maze[cur.row as usize][cur.col as usize]) != 0 {
            if finish_lock.win.is_none() {
                let _ = finish_lock.win.insert(guide.index);
            }
            drop(finish_lock);
            break;
        }
        finish_lock.maze[cur.row as usize][cur.col as usize] |= guide.paint;
        drop(finish_lock);

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {row: cur.row + p.row, col: cur.col + p.col};
            let seen_next = seen.contains_key(&next);
            let search_lock = monitor.lock().unwrap();
            let push_next: bool
                = !seen_next
                && (search_lock.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0;
            drop(search_lock);

            if push_next {
                seen.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve_util::NUM_DIRECTIONS;
            i != guide.index
        } {}
    }
    let mut paths_lock = paths.lock().unwrap();
    cur = *seen.get(&cur).unwrap();
    while cur.row > 0 {
       paths_lock[guide.index].push(cur);
       cur = *seen.get(&cur).unwrap();
    }
}
