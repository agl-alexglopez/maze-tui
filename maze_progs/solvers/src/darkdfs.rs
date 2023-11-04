use crate::solve;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::{thread, time};

// Public Solver Functions-------------------------------------------------------------------------

pub fn animate_hunt(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    if monitor
        .solver
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_hunt(monitor, speed);
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        all_start
    } else {
        print::maze_panic!("Thread panick.");
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

fn animate_mini_hunt(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        all_start
    } else {
        print::maze_panic!("Thread panick.");
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

pub fn animate_gather(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    if monitor
        .solver
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_gather(monitor, speed);
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        }
        all_start
    } else {
        print::maze_panic!("Thread panick.");
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

fn animate_mini_gather(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        }
        all_start
    } else {
        print::maze_panic!("Thread panick.");
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

pub fn animate_corner(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    if monitor
        .solver
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_corner(monitor, speed);
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.solver.lock() {
        solve::deluminate_maze(&lk.maze);
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            *lk.maze.get_mut(p.row, p.col) |= solve::START_BIT;
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
            *lk.maze.get_mut(next.row, next.col) |= maze::PATH_BIT;
        }
        *lk.maze.get_mut(finish.row, finish.col) |= maze::PATH_BIT;
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        corner_starts
    } else {
        print::maze_panic!("Thread panick.");
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

fn animate_mini_corner(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.solver.lock() {
        solve::deluminate_maze(&lk.maze);
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            *lk.maze.get_mut(p.row, p.col) |= solve::START_BIT;
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
            *lk.maze.get_mut(next.row, next.col) |= maze::PATH_BIT;
        }
        *lk.maze.get_mut(finish.row, finish.col) |= maze::PATH_BIT;
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        corner_starts
    } else {
        print::maze_panic!("Thread panick.");
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

fn animated_hunter(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            if lk.win.is_some() {
                return;
            }
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
                lk.win.get_or_insert(guide.index);
                dfs.pop();
                return;
            }
            *lk.maze.get_mut(cur.row, cur.col) |= seen | guide.paint;
            solve::flush_cursor_path_coordinate(&lk.maze, cur);
        } else {
            print::maze_panic!("Solve thread panic!");
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

            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze.get(next.row, next.col) & seen) == 0
                        && (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }

        match monitor.solver.lock() {
            Ok(mut lk) => {
                *lk.maze.get_mut(cur.row, cur.col) &= !guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }
        thread::sleep(time::Duration::from_micros(guide.speed));
        dfs.pop();
    }
}

fn animated_mini_hunter(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            if lk.win.is_some() {
                return;
            }
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
                solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
                lk.win.get_or_insert(guide.index);
                dfs.pop();
                return;
            }
            *lk.maze.get_mut(cur.row, cur.col) |= seen | guide.paint;
            solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
        } else {
            print::maze_panic!("Solve thread panic!");
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

            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze.get(next.row, next.col) & seen) == 0
                        && (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }

        match monitor.solver.lock() {
            Ok(mut lk) => {
                *lk.maze.get_mut(cur.row, cur.col) &= !guide.paint;
                solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }
        thread::sleep(time::Duration::from_micros(guide.speed));
        dfs.pop();
    }
}

fn animated_gatherer(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            match (
                (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0,
                (lk.maze.get(cur.row, cur.col) & solve::CACHE_MASK) == 0,
            ) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                    return;
                }
                (true, false) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen;
                }
                (_, _) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                }
            }
        } else {
            print::maze_panic!("Solve thread panic!");
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

            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze.get(next.row, next.col) & seen) == 0
                        && (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }

            i = (i + 1) % solve::NUM_DIRECTIONS;
        }

        match monitor.solver.lock() {
            Ok(mut lk) => {
                *lk.maze.get_mut(cur.row, cur.col) &= !guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }
        thread::sleep(time::Duration::from_micros(guide.speed));
        dfs.pop();
    }
}

fn animated_mini_gatherer(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            match (
                (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0,
                (lk.maze.get(cur.row, cur.col) & solve::CACHE_MASK) == 0,
            ) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen;
                    solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
                    return;
                }
                (true, false) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen;
                }
                (_, _) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen;
                    solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
                }
            }
        } else {
            print::maze_panic!("Solve thread panic!");
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

            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Solve thread panic: {}", p),
                Ok(lk) => {
                    (lk.maze.get(next.row, next.col) & seen) == 0
                        && (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0
                }
            } {
                dfs.push(next);
                continue 'branching;
            }

            i = (i + 1) % solve::NUM_DIRECTIONS;
        }

        match monitor.solver.lock() {
            Ok(mut lk) => {
                *lk.maze.get_mut(cur.row, cur.col) &= !guide.paint;
                solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
            }
            Err(p) => print::maze_panic!("Solve thread panic!: {}", p),
        }
        thread::sleep(time::Duration::from_micros(guide.speed));
        dfs.pop();
    }
}
