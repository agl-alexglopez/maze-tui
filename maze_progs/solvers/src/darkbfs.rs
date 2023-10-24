use crate::solve;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::{thread, time};

// Public Solver Functions-------------------------------------------------------------------------

pub fn animate_hunt(monitor: solve::SolverMonitor, speed: speed::Speed) {
    if monitor
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_hunt(monitor, speed);
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        all_start
    } else {
        print::maze_panic!("Thread panick.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                monitor_clone,
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

    if let Ok(mut lk) = monitor.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
            lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
            solve::flush_cursor_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
}

fn animate_mini_hunt(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        all_start
    } else {
        print::maze_panic!("Thread panick.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_mini_hunter(
                monitor_clone,
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

    if let Ok(mut lk) = monitor.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
            lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
            solve::flush_dark_mini_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
}

pub fn animate_gather(monitor: solve::SolverMonitor, speed: speed::Speed) {
    if monitor
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_gather(monitor, speed);
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        }
        all_start
    } else {
        print::maze_panic!("Thread panick.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_gatherer(
                monitor_clone,
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

    if let Ok(mut lk) = monitor.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
            lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
            solve::flush_cursor_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

fn animate_mini_gather(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        solve::deluminate_maze(&lk.maze);
        let all_start = solve::pick_random_point(&lk.maze);
        lk.maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        }
        all_start
    } else {
        print::maze_panic!("Thread panick.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_mini_gatherer(
                monitor_clone,
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

    if let Ok(mut lk) = monitor.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
            lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
            solve::flush_dark_mini_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

pub fn animate_corner(monitor: solve::SolverMonitor, speed: speed::Speed) {
    if monitor
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_corner(monitor, speed);
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut all_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        solve::deluminate_maze(&lk.maze);
        let all_starts = solve::set_corner_starts(&lk.maze);
        for s in all_starts {
            lk.maze[s.row as usize][s.col as usize] |= solve::START_BIT;
        }
        let finish = maze::Point {
            row: lk.maze.row_size() / 2,
            col: lk.maze.col_size() / 2,
        };
        for p in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + p.row,
                col: finish.col + p.col,
            };
            lk.maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
        }
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        lk.maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
        all_starts
    } else {
        print::maze_panic!("Thread panick.");
    };

    all_starts.shuffle(&mut thread_rng());

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                monitor_clone,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_starts[i_thread + 1],
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
            start: all_starts[0],
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }

    if let Ok(mut lk) = monitor.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
            lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
            solve::flush_cursor_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

fn animate_mini_corner(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut all_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        if lk.maze.exit() {
            return;
        }
        solve::deluminate_maze(&lk.maze);
        let all_starts = solve::set_corner_starts(&lk.maze);
        for s in all_starts {
            lk.maze[s.row as usize][s.col as usize] |= solve::START_BIT;
        }
        let finish = maze::Point {
            row: lk.maze.row_size() / 2,
            col: lk.maze.col_size() / 2,
        };
        for p in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + p.row,
                col: finish.col + p.col,
            };
            lk.maze[next.row as usize][next.col as usize] |= maze::PATH_BIT;
        }
        lk.maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
        lk.maze[finish.row as usize][finish.col as usize] |= maze::PATH_BIT;
        all_starts
    } else {
        print::maze_panic!("Thread panick.");
    };

    all_starts.shuffle(&mut thread_rng());

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_mini_hunter(
                monitor_clone,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_starts[i_thread + 1],
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
            start: all_starts[0],
            speed: animation,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }

    if let Ok(mut lk) = monitor.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            lk.maze[p.0.row as usize][p.0.col as usize] &= !solve::THREAD_MASK;
            lk.maze[p.0.row as usize][p.0.col as usize] |= p.1;
            solve::flush_dark_mini_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn animated_hunter(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(mut cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() || lk.win.is_some() {
                return;
            }
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
        } else {
            print::maze_panic!("Thread panicked!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
    }
}

fn animated_mini_hunter(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(mut cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() || lk.win.is_some() {
                return;
            }
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
            solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
        } else {
            print::maze_panic!("Thread panicked!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
    }
}

fn animated_gatherer(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() {
                return;
            }
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
        } else {
            print::maze_panic!("Thread panicked!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
    }
}

fn animated_mini_gatherer(monitor: solve::SolverMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.maze.exit() {
                return;
            }
            let finish = (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0;
            let first = (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0;
            // We can only stop looking if we are the first to this finish. Keep looking otherwise.
            match (finish, first) {
                (true, true) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= guide.paint | seen_bit;
                    solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
                    return;
                }
                (true, false) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen_bit;
                }
                (_, _) => {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen_bit | guide.paint;
                    solve::flush_dark_mini_path_coordinate(&lk.maze, cur);
                }
            }
        } else {
            print::maze_panic!("Thread panicked!");
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
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
        }
    }
}
