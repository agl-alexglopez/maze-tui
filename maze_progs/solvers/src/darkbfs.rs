use crate::solve;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::{thread, time};

struct BfsSolver {
    maze: maze::BoxMaze,
    win: Option<usize>,
    win_path: Vec<(maze::Point, solve::ThreadPaint)>,
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

pub fn animate_hunt(mut maze: maze::BoxMaze, speed: speed::Speed) {
    solve::print_overlap_key(&maze);
    solve::deluminate_maze(&maze);
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
    let finish: maze::Point = solve::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().enumerate() {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    match monitor.lock() {
        Ok(mut lk) => {
            for i in 0..lk.win_path.len() {
                let p = lk.win_path[i];
                lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
                lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
                solve::flush_cursor_path_coordinate(&lk.maze, p.0);
                thread::sleep(time::Duration::from_micros(animation));
            }
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn animate_gather(mut maze: maze::BoxMaze, speed: speed::Speed) {
    solve::print_overlap_key(&maze);
    solve::deluminate_maze(&maze);
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;

    for _ in 0..solve::NUM_GATHER_FINISHES {
        let finish: maze::Point = solve::pick_random_point(&maze);
        maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    }

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().enumerate() {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_gatherer(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    match monitor.lock() {
        Ok(mut lk) => {
            for i in 0..lk.win_path.len() {
                let p = lk.win_path[i];
                lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
                lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
                solve::flush_cursor_path_coordinate(&lk.maze, p.0);
                thread::sleep(time::Duration::from_micros(animation));
            }
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn animate_corner(mut maze: maze::BoxMaze, speed: speed::Speed) {
    solve::print_overlap_key(&maze);
    solve::deluminate_maze(&maze);
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut all_starts: [maze::Point; 4] = solve::set_corner_starts(&maze);
    for s in all_starts {
        maze[s.row as usize][s.col as usize] |= solve::START_BIT;
    }
    let finish = maze::Point {
        row: maze.row_size() / 2,
        col: maze.col_size() / 2,
    };
    for p in maze::ALL_DIRECTIONS {
        let next = maze::Point {
            row: finish.row + p.row,
            col: finish.col + p.col,
        };
        maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
    }
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;

    all_starts.shuffle(&mut thread_rng());

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().enumerate() {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: all_starts[i_thread],
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    match monitor.lock() {
        Ok(mut lk) => {
            for i in 0..lk.win_path.len() {
                let p = lk.win_path[i];
                lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
                lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
                solve::flush_cursor_path_coordinate(&lk.maze, p.0);
                thread::sleep(time::Duration::from_micros(animation));
            }
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn animated_hunter(monitor: &mut BfsMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(mut cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => match lk.win {
                Some(_) => return,
                None => {
                    if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                        lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                        lk.win.get_or_insert(guide.index);
                        while cur.row > 0 {
                            lk.win_path.push((cur, guide.paint));
                            cur = match parents.get(&cur) {
                                Some(parent) => *parent,
                                None => print::maze_panic!("Bfs could not find parent."),
                            };
                        }
                        return;
                    }
                    lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                }
            },
            Err(p) => {
                print::maze_panic!("Thread panicked: {}", p);
            }
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
    }
}

fn animated_gatherer(monitor: &mut BfsMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => {
                let finish = (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0;
                let first = (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0;
                // We can only stop looking if we are the first to this finish. Keep looking otherwise.
                match (finish, first) {
                    (true, true) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= guide.paint | seen_bit;
                        solve::flush_cursor_path_coordinate(&lk.maze, cur);
                        return;
                    }
                    (true, false) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= seen_bit;
                    }
                    (_, _) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= seen_bit | guide.paint;
                        solve::flush_cursor_path_coordinate(&lk.maze, cur);
                    }
                }
            }
            Err(p) => print::maze_panic!("Thread panicked: {}", p),
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
    }
}
