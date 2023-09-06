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

pub fn hunt(mut maze: maze::BoxMaze) {
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
    let finish: maze::Point = solve::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;

    let monitor = BfsSolver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().enumerate() {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: all_start,
                    speed: 0,
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
            }
            solve::print_paths(&lk.maze);
            solve::print_overlap_key();
            solve::print_hunt_solution_message(lk.win);
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn animate_hunt(mut maze: maze::BoxMaze, speed: speed::Speed) {
    print::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve::print_overlap_key();
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
    let finish: maze::Point = solve::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;

    solve::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

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
            print::set_cursor_position(maze::Point {
                row: lk.maze.row_size() + solve::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
                col: 0,
            });
            solve::print_hunt_solution_message(lk.win);
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn gather(mut maze: maze::BoxMaze) {
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
            gatherer(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: all_start,
                    speed: 0,
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
            }

            solve::print_paths(&lk.maze);
            solve::print_overlap_key();
            solve::print_gather_solution_message();
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn animate_gather(mut maze: maze::BoxMaze, speed: speed::Speed) {
    print::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve::print_overlap_key();
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;

    for _ in 0..solve::NUM_GATHER_FINISHES {
        let finish: maze::Point = solve::pick_random_point(&maze);
        maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        solve::flush_cursor_path_coordinate(&maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
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
            print::set_cursor_position(maze::Point {
                row: lk.maze.row_size() + solve::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
                col: 0,
            });
            solve::print_gather_solution_message();
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn corner(mut maze: maze::BoxMaze) {
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
            hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: all_starts[i_thread],
                    speed: 0,
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
            }

            solve::print_paths(&lk.maze);
            solve::print_overlap_key();
            solve::print_hunt_solution_message(lk.win);
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

pub fn animate_corner(mut maze: maze::BoxMaze, speed: speed::Speed) {
    print::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve::print_overlap_key();
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut all_starts: [maze::Point; 4] = solve::set_corner_starts(&maze);
    for s in all_starts {
        maze[s.row as usize][s.col as usize] |= solve::START_BIT;
        solve::flush_cursor_path_coordinate(&maze, s);
        thread::sleep(time::Duration::from_micros(animation));
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
        solve::flush_cursor_path_coordinate(&maze, next);
        thread::sleep(time::Duration::from_micros(animation));
    }
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
    solve::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

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
            print::set_cursor_position(maze::Point {
                row: lk.maze.row_size() + solve::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
                col: 0,
            });
            solve::print_hunt_solution_message(lk.win);
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked with the lock: {}", p),
    };
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn hunter(monitor: &mut BfsMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => match lk.win {
                Some(_) => return,
                None => {
                    if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                        lk.win.get_or_insert(guide.index);
                        let mut prev = match parents.get(&cur) {
                            Some(p) => p,
                            None => print::maze_panic!("Bfs could not find parent."),
                        };
                        while prev.row > 0 {
                            lk.win_path.push((*prev, guide.paint));
                            prev = match parents.get(prev) {
                                Some(parent) => parent,
                                None => print::maze_panic!("Bfs could not find parent."),
                            };
                        }
                        return;
                    }
                    lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                }
            },
            Err(p) => print::maze_panic!("Thread panicked: {}", p),
        };

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } && !parents.contains_key(&next) {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
    }
}

fn animated_hunter(monitor: &mut BfsMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => match lk.win {
                Some(_) => return,
                None => {
                    if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                        lk.win.get_or_insert(guide.index);
                        let mut prev = match parents.get(&cur) {
                            Some(p) => p,
                            None => print::maze_panic!("Bfs could not find parent."),
                        };
                        while prev.row > 0 {
                            lk.win_path.push((*prev, guide.paint));
                            prev = match parents.get(prev) {
                                Some(parent) => parent,
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
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } && !parents.contains_key(&next) {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
    }
}

fn gatherer(monitor: &mut BfsMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => {
                if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0
                    && (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0
                {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen_bit;
                    let p = match parents.get(&cur) {
                        Some(point) => {
                            point
                        }
                        None => print::maze_panic!("Could not find parent to maze point in bfs."),
                    };
                    lk.win_path.push((*p, guide.paint));
                    return;
                }
                lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
            }
            Err(p) => print::maze_panic!("Thread panicked: {}", p),
        }

        let mut i = guide.index;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } && !parents.contains_key(&next) {
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
                if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0
                    && (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0
                {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen_bit;
                    let p = match parents.get(&cur) {
                        Some(point) => {
                            point
                        }
                        None => print::maze_panic!("Could not find parent to maze point in bfs."),
                    };
                    lk.win_path.push((*p, guide.paint));
                    return;
                }
                lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
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
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } && !parents.contains_key(&next) {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
    }
}
