use crate::solve;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::{thread, time};

// Public Solver Functions-------------------------------------------------------------------------

pub fn hunt(mut maze: maze::BoxMaze) {
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
    let finish: maze::Point = solve::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;

    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
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
        Ok(print_lock) => {
            solve::print_paths(&print_lock.maze);
            solve::print_overlap_key(&print_lock.maze);
        }
        Err(p) => print::maze_panic!("Solve thread print::maze_panic! somehow: {}", p),
    };
}

pub fn gather(mut maze: maze::BoxMaze) {
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;

    for _ in 0..solve::NUM_GATHER_FINISHES {
        let finish: maze::Point = solve::pick_random_point(&maze);
        maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    }

    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
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
        Ok(print_lock) => {
            solve::print_paths(&print_lock.maze);
            solve::print_overlap_key(&print_lock.maze);
        }
        Err(p) => print::maze_panic!("Solve thread print::maze_panic! somehow: {}", p),
    };
}

pub fn corner(mut maze: maze::BoxMaze) {
    let mut corner_starts: [maze::Point; 4] = solve::set_corner_starts(&maze);
    for p in corner_starts {
        maze[p.row as usize][p.col as usize] |= solve::START_BIT;
    }

    let finish = maze::Point {
        row: maze.row_size() / 2,
        col: maze.col_size() / 2,
    };
    for d in maze::ALL_DIRECTIONS {
        let next = maze::Point {
            row: finish.row + d.row,
            col: finish.col + d.col,
        };
        maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
    }
    maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;

    corner_starts.shuffle(&mut thread_rng());
    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().enumerate() {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: corner_starts[i_thread],
                    speed: 0,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    match monitor.lock() {
        Ok(print_lock) => {
            solve::print_paths(&print_lock.maze);
            solve::print_overlap_key(&print_lock.maze);
        }
        Err(p) => print::maze_panic!("Solve thread print::maze_panic!: {}", p),
    };
}

pub fn animate_hunt(mut maze: maze::BoxMaze, speed: speed::Speed) {
    solve::print_overlap_key(&maze);
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
    let finish: maze::Point = solve::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    solve::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
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
}

pub fn animate_gather(mut maze: maze::BoxMaze, speed: speed::Speed) {
    solve::print_overlap_key(&maze);

    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;

    for _ in 0..solve::NUM_GATHER_FINISHES {
        let finish: maze::Point = solve::pick_random_point(&maze);
        maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        solve::flush_cursor_path_coordinate(&maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
    }

    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
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
}

pub fn animate_corner(mut maze: maze::BoxMaze, speed: speed::Speed) {
    solve::print_overlap_key(&maze);
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut corner_starts: [maze::Point; 4] = solve::set_corner_starts(&maze);
    for p in corner_starts {
        maze[p.row as usize][p.col as usize] |= solve::START_BIT;
        solve::flush_cursor_path_coordinate(&maze, p);
        thread::sleep(time::Duration::from_micros(animation));
    }

    let finish = maze::Point {
        row: maze.row_size() / 2,
        col: maze.col_size() / 2,
    };
    for d in maze::ALL_DIRECTIONS {
        let next = maze::Point {
            row: finish.row + d.row,
            col: finish.col + d.col,
        };
        maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
        solve::flush_cursor_path_coordinate(&maze, next);
        thread::sleep(time::Duration::from_micros(animation));
    }
    maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    solve::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    corner_starts.shuffle(&mut thread_rng());
    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().enumerate() {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: mask,
                    start: corner_starts[i_thread],
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn hunter(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);

    'branching: while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => match lk.win {
                Some(_) => {
                    for p in dfs {
                        lk.maze[p.row as usize][p.col as usize] |= guide.paint;
                    }
                    return;
                }
                None => {
                    if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                        lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                        lk.win.get_or_insert(guide.index);
                        for p in dfs {
                            if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT)
                                == 0
                            {
                                lk.maze[p.row as usize][p.col as usize] |= guide.paint;
                            }
                        }
                        return;
                    }
                    lk.maze[cur.row as usize][cur.col as usize] |= seen;
                }
            },
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        };

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        while {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            if match monitor.lock() {
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
            } {
                dfs.push(next);
                continue 'branching;
            }

            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
        dfs.pop();
    }
}

fn animated_hunter(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => match lk.win {
                Some(_) => return,
                None => {
                    if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                        lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                        solve::flush_cursor_path_coordinate(&lk.maze, cur);
                        lk.win.get_or_insert(guide.index);
                        return;
                    }
                    lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                }
            },
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        };

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        while {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            if match monitor.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }

            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}

        match monitor.lock() {
            Ok(mut lk) => {
                lk.maze[cur.row as usize][cur.col as usize] &= !guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }
        thread::sleep(time::Duration::from_micros(guide.speed));
        dfs.pop();
    }
}

fn gatherer(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => {
                match (
                    (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0,
                    (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0,
                ) {
                    (true, true) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                        for p in dfs {
                            if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT)
                                == 0
                            {
                                lk.maze[p.row as usize][p.col as usize] |= guide.paint;
                            }
                        }
                        return;
                    }
                    (true, false) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= seen;
                    }
                    (_, _) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                    }
                }
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        };

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        while {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }

            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}
        dfs.pop();
    }
}

fn animated_gatherer(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => {
                match (
                    (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0,
                    (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0,
                ) {
                    (true, true) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= guide.paint | seen;
                        solve::flush_cursor_path_coordinate(&lk.maze, cur);
                        return;
                    }
                    (true, false) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= seen;
                    }
                    (_, _) => {
                        lk.maze[cur.row as usize][cur.col as usize] |= guide.paint | seen;
                        solve::flush_cursor_path_coordinate(&lk.maze, cur);
                    }
                }
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        while {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            if match monitor.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }

            i = (i + 1) % solve::NUM_DIRECTIONS;
            i != guide.index
        } {}

        match monitor.lock() {
            Ok(mut lk) => {
                lk.maze[cur.row as usize][cur.col as usize] &= !guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }
        thread::sleep(time::Duration::from_micros(guide.speed));
        dfs.pop();
    }
}
