use crate::solve;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::{thread, time};

// Public Solver Functions-------------------------------------------------------------------------

pub fn hunt(monitor: solve::SolverMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: 0,
                },
            );
        }));
    }
    hunter(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: all_start,
            speed: 0,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
    if let Ok(lk) = monitor.lock() {
        solve::print_paths(&lk.maze);
        return;
    }
    print::maze_panic!("Solve thread print::maze_panic!");
}

pub fn gather(monitor: solve::SolverMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        }
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            gatherer(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: 0,
                },
            );
        }));
    }
    gatherer(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: all_start,
            speed: 0,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
    if let Ok(lk) = monitor.lock() {
        solve::print_paths(&lk.maze);
        return;
    }
    print::maze_panic!("Solve thread print::maze_panic!");
}

pub fn corner(monitor: solve::SolverMonitor) {
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            lk.maze[p.row as usize][p.col as usize] |= solve::START_BIT;
        }
        let finish = maze::Point {
            row: lk.maze.row_size() / 2,
            col: lk.maze.col_size() / 2,
        };
        for d in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + d.row,
                col: finish.col + d.col,
            };
            lk.maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
        }
        lk.maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        corner_starts
    } else {
        print::maze_panic!("Thread panic.");
    };

    corner_starts.shuffle(&mut thread_rng());
    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: corner_starts[i_thread + 1],
                    speed: 0,
                },
            );
        }));
    }
    hunter(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: corner_starts[0],
            speed: 0,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
    if let Ok(lk) = monitor.lock() {
        solve::print_paths(&lk.maze);
        return;
    }
    print::maze_panic!("Solve thread print::maze_panic!");
}

pub fn animate_hunt(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        solve::flush_cursor_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    animated_hunter(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: all_start,
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn animate_mini_hunt(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        solve::flush_mini_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_mini_hunter(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    animated_mini_hunter(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: all_start,
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn animate_gather(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
            solve::flush_cursor_path_coordinate(&lk.maze, finish);
            thread::sleep(time::Duration::from_micros(animation));
        }
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_gatherer(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    animated_gatherer(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: all_start,
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn animate_mini_gather(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
            solve::flush_mini_path_coordinate(&lk.maze, finish);
            thread::sleep(time::Duration::from_micros(animation));
        }
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_mini_gatherer(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: animation,
                },
            );
        }));
    }
    animated_mini_gatherer(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: all_start,
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn animate_corner(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            lk.maze[p.row as usize][p.col as usize] |= solve::START_BIT;
            solve::flush_cursor_path_coordinate(&lk.maze, p);
            thread::sleep(time::Duration::from_micros(animation));
        }

        let finish = maze::Point {
            row: lk.maze.row_size() / 2,
            col: lk.maze.col_size() / 2,
        };
        for d in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + d.row,
                col: finish.col + d.col,
            };
            lk.maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
            solve::flush_cursor_path_coordinate(&lk.maze, next);
            thread::sleep(time::Duration::from_micros(animation));
        }
        lk.maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        solve::flush_cursor_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
        corner_starts
    } else {
        print::maze_panic!("Thread panic.");
    };

    corner_starts.shuffle(&mut thread_rng());
    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: corner_starts[i_thread + 1],
                    speed: animation,
                },
            );
        }));
    }
    animated_hunter(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: corner_starts[0],
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

pub fn animate_mini_corner(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            lk.maze[p.row as usize][p.col as usize] |= solve::START_BIT;
            solve::flush_mini_path_coordinate(&lk.maze, p);
            thread::sleep(time::Duration::from_micros(animation));
        }

        let finish = maze::Point {
            row: lk.maze.row_size() / 2,
            col: lk.maze.col_size() / 2,
        };
        for d in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + d.row,
                col: finish.col + d.col,
            };
            lk.maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
            solve::flush_mini_path_coordinate(&lk.maze, next);
            thread::sleep(time::Duration::from_micros(animation));
        }
        lk.maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        solve::flush_mini_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
        corner_starts
    } else {
        print::maze_panic!("Thread panic.");
    };

    corner_starts.shuffle(&mut thread_rng());
    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_mini_hunter(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: corner_starts[i_thread + 1],
                    speed: animation,
                },
            );
        }));
    }
    animated_mini_hunter(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            start: corner_starts[0],
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn hunter(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);

    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.win.is_some() {
                for p in dfs {
                    lk.maze[p.row as usize][p.col as usize] |= guide.paint;
                }
                return;
            }
            if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                lk.win.get_or_insert(guide.index);
                return;
            }
            lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
        dfs.pop();
    }
}

fn animated_hunter(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() || lk.win.is_some() {
                return;
            }
            if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
                lk.win.get_or_insert(guide.index);
                return;
            }
            lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
            solve::flush_cursor_path_coordinate(&lk.maze, cur);
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
        dfs.pop();
    }
}

fn animated_mini_hunter(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() || lk.win.is_some() {
                return;
            }
            if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                lk.maze[cur.row as usize][cur.col as usize] |= guide.paint;
                solve::flush_mini_path_coordinate(&lk.maze, cur);
                lk.win.get_or_insert(guide.index);
                return;
            }
            lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
            solve::flush_mini_path_coordinate(&lk.maze, cur);
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
        dfs.pop();
    }
}

fn gatherer(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            match (
                (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0,
                (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0,
            ) {
                (true, true) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                    return;
                }
                (true, false) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen;
                }
                (_, _) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                }
            }
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
        dfs.pop();
    }
}

fn animated_gatherer(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() {
                return;
            }
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
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
        dfs.pop();
    }
}

fn animated_mini_gatherer(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() {
                return;
            }
            match (
                (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0,
                (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0,
            ) {
                (true, true) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= guide.paint | seen;
                    solve::flush_mini_path_coordinate(&lk.maze, cur);
                    return;
                }
                (true, false) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen;
                }
                (_, _) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= guide.paint | seen;
                    solve::flush_mini_path_coordinate(&lk.maze, cur);
                }
            }
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
        dfs.pop();
    }
}
