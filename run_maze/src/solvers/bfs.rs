use crate::maze;
use crate::utilities::print_util;
use crate::utilities::solve_util;

use rand::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::{thread, time};

struct BfsSolver {
    maze: maze::BoxMaze,
    win: Option<usize>,
    win_path: Vec<(maze::Point, solve_util::ThreadPaint)>,
}

impl BfsSolver {
    fn new(boxed_maze: maze::BoxMaze) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            maze: boxed_maze,
            win: None,
            win_path: Vec::new(),
        }))
    }
}

type BfsMonitor = Arc<Mutex<BfsSolver>>;

// Public Solver Functions-------------------------------------------------------------------------

pub fn solve_with_bfs_thread_hunt(mut maze: maze::BoxMaze) {
    let all_start: maze::Point = solve_util::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            complete_hunt(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_start,
                    speed: 0,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut show_winner = monitor.lock().unwrap();
    for i in 0..show_winner.win_path.len() {
        let p = show_winner.win_path[i];
        show_winner.maze[p.0.row as usize][p.0.col as usize] &= !solve_util::THREAD_MASK;
        show_winner.maze[p.0.row as usize][p.0.col as usize] |= p.1;
    }

    solve_util::print_paths(&show_winner.maze);
    solve_util::print_overlap_key();
    solve_util::print_hunt_solution_message(show_winner.win);
    println!();
}

pub fn animate_with_bfs_thread_hunt(mut maze: maze::BoxMaze, speed: solve_util::SolverSpeed) {
    print_util::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve_util::print_overlap_key();
    let animation = solve_util::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve_util::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve_util::START_BIT;
    let finish: maze::Point = solve_util::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;

    solve_util::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animate_hunt(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut show_winner = monitor.lock().unwrap();
    for i in 0..show_winner.win_path.len() {
        let p = show_winner.win_path[i];
        show_winner.maze[p.0.row as usize][p.0.col as usize] &= !solve_util::THREAD_MASK;
        show_winner.maze[p.0.row as usize][p.0.col as usize] |= p.1;
        solve_util::flush_cursor_path_coordinate(&show_winner.maze, p.0);
        thread::sleep(time::Duration::from_micros(animation));
    }
    print_util::set_cursor_position(maze::Point {
        row: show_winner.maze.row_size() + solve_util::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
        col: 0,
    });
    solve_util::print_hunt_solution_message(show_winner.win);
    println!();
}

pub fn solve_with_bfs_thread_gather(mut maze: maze::BoxMaze) {
    let all_start: maze::Point = solve_util::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve_util::START_BIT;

    for _ in 0..solve_util::NUM_GATHER_FINISHES {
        let finish: maze::Point = solve_util::pick_random_point(&maze);
        maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    }

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            complete_gather(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_start,
                    speed: 0,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut show_finishes = monitor.lock().unwrap();
    for i in 0..show_finishes.win_path.len() {
        let p = show_finishes.win_path[i];
        show_finishes.maze[p.0.row as usize][p.0.col as usize] &= !solve_util::THREAD_MASK;
        show_finishes.maze[p.0.row as usize][p.0.col as usize] |= p.1;
    }

    solve_util::print_paths(&show_finishes.maze);
    solve_util::print_overlap_key();
    solve_util::print_gather_solution_message();
    println!();
}

pub fn animate_with_bfs_thread_gather(mut maze: maze::BoxMaze, speed: solve_util::SolverSpeed) {
    print_util::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve_util::print_overlap_key();
    let animation = solve_util::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve_util::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve_util::START_BIT;

    for _ in 0..solve_util::NUM_GATHER_FINISHES {
        let finish: maze::Point = solve_util::pick_random_point(&maze);
        maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
        solve_util::flush_cursor_path_coordinate(&maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
    }

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animate_gather(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut show_finishes = monitor.lock().unwrap();
    for i in 0..show_finishes.win_path.len() {
        let p = show_finishes.win_path[i];
        show_finishes.maze[p.0.row as usize][p.0.col as usize] &= !solve_util::THREAD_MASK;
        show_finishes.maze[p.0.row as usize][p.0.col as usize] |= p.1;
        solve_util::flush_cursor_path_coordinate(&show_finishes.maze, p.0);
        thread::sleep(time::Duration::from_micros(animation));
    }
    print_util::set_cursor_position(maze::Point {
        row: show_finishes.maze.row_size() + solve_util::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
        col: 0,
    });
    solve_util::print_gather_solution_message();
    println!();
}

pub fn solve_with_bfs_thread_corners(mut maze: maze::BoxMaze) {
    let mut all_starts: [maze::Point; 4] = solve_util::set_corner_starts(&maze);
    for s in all_starts {
        maze[s.row as usize][s.col as usize] |= solve_util::START_BIT;
    }
    let finish = maze::Point {row: maze.row_size() / 2, col: maze.col_size() / 2};
    for p in maze::ALL_DIRECTIONS {
        let next = maze::Point {row: finish.row + p.row, col: finish.col + p.col};
        maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
    }
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;

    all_starts.shuffle(&mut thread_rng());
    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            complete_hunt(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_starts[i_thread],
                    speed: 0,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut show_winner = monitor.lock().unwrap();
    for i in 0..show_winner.win_path.len() {
        let p = show_winner.win_path[i];
        show_winner.maze[p.0.row as usize][p.0.col as usize] &= !solve_util::THREAD_MASK;
        show_winner.maze[p.0.row as usize][p.0.col as usize] |= p.1;
    }

    solve_util::print_paths(&show_winner.maze);
    solve_util::print_overlap_key();
    solve_util::print_hunt_solution_message(show_winner.win);
    println!();
}

pub fn animate_with_bfs_thread_corners(mut maze: maze::BoxMaze, speed: solve_util::SolverSpeed) {
    print_util::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve_util::print_overlap_key();
    let animation = solve_util::SOLVER_SPEEDS[speed as usize];
    let mut all_starts: [maze::Point; 4] = solve_util::set_corner_starts(&maze);
    for s in all_starts {
        maze[s.row as usize][s.col as usize] |= solve_util::START_BIT;
        solve_util::flush_cursor_path_coordinate(&maze, s);
        thread::sleep(time::Duration::from_micros(animation));
    }
    let finish = maze::Point {row: maze.row_size() / 2, col: maze.col_size() / 2};
    for p in maze::ALL_DIRECTIONS {
        let next = maze::Point {row: finish.row + p.row, col: finish.col + p.col};
        maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
        solve_util::flush_cursor_path_coordinate(&maze, next);
        thread::sleep(time::Duration::from_micros(animation));
    }
    maze[finish.row as usize][finish.col as usize] |= solve_util::FINISH_BIT;
    maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
    solve_util::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    all_starts.shuffle(&mut thread_rng());

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve_util::NUM_THREADS);
    for i_thread in 0..solve_util::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animate_hunt(
                &mut monitor_clone,
                solve_util::ThreadGuide {
                    index: i_thread,
                    paint: solve_util::THREAD_MASKS[i_thread],
                    start: all_starts[i_thread],
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut show_winner = monitor.lock().unwrap();
    for i in 0..show_winner.win_path.len() {
        let p = show_winner.win_path[i];
        show_winner.maze[p.0.row as usize][p.0.col as usize] &= !solve_util::THREAD_MASK;
        show_winner.maze[p.0.row as usize][p.0.col as usize] |= p.1;
        solve_util::flush_cursor_path_coordinate(&show_winner.maze, p.0);
        thread::sleep(time::Duration::from_micros(animation));
    }
    print_util::set_cursor_position(maze::Point {
        row: show_winner.maze.row_size() + solve_util::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
        col: 0,
    });
    solve_util::print_hunt_solution_message(show_winner.win);
    println!();
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn complete_hunt(monitor: &mut BfsMonitor, guide: solve_util::ThreadGuide) {
    let mut seen: HashMap<maze::Point, maze::Point> = HashMap::new();
    seen.insert(guide.start, maze::Point { row: -1, col: -1 });
    let mut bfs: VecDeque<maze::Point> = VecDeque::new();
    bfs.push_back(guide.start);

    while !bfs.is_empty() {
        let win_lock = monitor.lock().unwrap();
        if win_lock.win.is_some() {
            return;
        }
        drop(win_lock);
        let mut cur = bfs.pop_front().unwrap();

        let mut finish_lock = monitor.lock().unwrap();
        if (finish_lock.maze[cur.row as usize][cur.col as usize] & solve_util::FINISH_BIT) != 0 {
            match finish_lock.win {
                Some(_) => return,
                None => {
                    let _ = finish_lock.win.insert(guide.index);
                    cur = *seen.get(&cur).unwrap();
                    while cur.row > 0 {
                        finish_lock.win_path.push((cur, guide.paint));
                        cur = *seen.get(&cur).unwrap();
                    }
                    return;
                }
            }
        }
        finish_lock.maze[cur.row as usize][cur.col as usize] |= guide.paint;
        drop(finish_lock);

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            let seen_next = seen.contains_key(&next);
            let search_lock = monitor.lock().unwrap();
            let push_next: bool = !seen_next
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
}

fn animate_hunt(monitor: &mut BfsMonitor, guide: solve_util::ThreadGuide) {
    let mut seen: HashMap<maze::Point, maze::Point> = HashMap::new();
    seen.insert(guide.start, maze::Point { row: -1, col: -1 });
    let mut bfs: VecDeque<maze::Point> = VecDeque::new();
    bfs.push_back(guide.start);

    while !bfs.is_empty() {
        let win_lock = monitor.lock().unwrap();
        if win_lock.win.is_some() {
            return;
        }
        drop(win_lock);
        let mut cur = bfs.pop_front().unwrap();

        let mut finish_lock = monitor.lock().unwrap();
        if (finish_lock.maze[cur.row as usize][cur.col as usize] & solve_util::FINISH_BIT) != 0 {
            match finish_lock.win {
                Some(_) => return,
                None => {
                    let _ = finish_lock.win.insert(guide.index);
                    cur = *seen.get(&cur).unwrap();
                    while cur.row > 0 {
                        finish_lock.win_path.push((cur, guide.paint));
                        cur = *seen.get(&cur).unwrap();
                    }
                    return;
                }
            }
        }
        finish_lock.maze[cur.row as usize][cur.col as usize] |= guide.paint;
        solve_util::flush_cursor_path_coordinate(&finish_lock.maze, cur);
        drop(finish_lock);
        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            let seen_next = seen.contains_key(&next);
            let search_lock = monitor.lock().unwrap();
            let push_next: bool = !seen_next
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
}

fn complete_gather(monitor: &mut BfsMonitor, guide: solve_util::ThreadGuide) {
    let mut seen: HashMap<maze::Point, maze::Point> = HashMap::new();
    let seen_bit: solve_util::ThreadCache = guide.paint << 4;
    seen.insert(guide.start, maze::Point { row: -1, col: -1 });
    let mut bfs: VecDeque<maze::Point> = VecDeque::new();
    bfs.push_back(guide.start);

    while !bfs.is_empty() {
        let cur = bfs.pop_front().unwrap();

        let mut finish_lock = monitor.lock().unwrap();
        if (finish_lock.maze[cur.row as usize][cur.col as usize] & solve_util::FINISH_BIT) != 0
            && (finish_lock.maze[cur.row as usize][cur.col as usize] & solve_util::CACHE_MASK) == 0
        {
            finish_lock.maze[cur.row as usize][cur.col as usize] |= seen_bit;
            finish_lock
                .win_path
                .push((*seen.get(&cur).unwrap(), guide.paint));
            return;
        }
        finish_lock.maze[cur.row as usize][cur.col as usize] |= guide.paint;
        drop(finish_lock);

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            let seen_next = seen.contains_key(&next);
            let search_lock = monitor.lock().unwrap();
            let push_next: bool = !seen_next
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
}

fn animate_gather(monitor: &mut BfsMonitor, guide: solve_util::ThreadGuide) {
    let mut seen: HashMap<maze::Point, maze::Point> = HashMap::new();
    let seen_bit: solve_util::ThreadCache = guide.paint << 4;
    seen.insert(guide.start, maze::Point { row: -1, col: -1 });
    let mut bfs: VecDeque<maze::Point> = VecDeque::new();
    bfs.push_back(guide.start);

    while !bfs.is_empty() {
        let cur = bfs.pop_front().unwrap();

        let mut finish_lock = monitor.lock().unwrap();
        if (finish_lock.maze[cur.row as usize][cur.col as usize] & solve_util::FINISH_BIT) != 0
            && (finish_lock.maze[cur.row as usize][cur.col as usize] & solve_util::CACHE_MASK) == 0
        {
            finish_lock.maze[cur.row as usize][cur.col as usize] |= seen_bit;
            finish_lock
                .win_path
                .push((*seen.get(&cur).unwrap(), guide.paint));
            return;
        }
        finish_lock.maze[cur.row as usize][cur.col as usize] |= guide.paint;
        solve_util::flush_cursor_path_coordinate(&finish_lock.maze, cur);
        drop(finish_lock);
        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            let seen_next = seen.contains_key(&next);
            let search_lock = monitor.lock().unwrap();
            let push_next: bool = !seen_next
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
}
