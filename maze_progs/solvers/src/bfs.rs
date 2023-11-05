use crate::solve;
use maze;
use print;
use speed;

use rand::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::{thread, time};

// Public Solver Functions-------------------------------------------------------------------------

pub fn hunt_history(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let start = solve::pick_random_point(&lk.maze);
        let start_square = lk.maze.get(start.row, start.col);
        lk.maze.solve_history.push(tape::Delta {
            id: start,
            before: start_square,
            after: start_square | solve::START_BIT,
            burst: 1,
        });
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        let finish_square = lk.maze.get(finish.row, finish.col);
        lk.maze.solve_history.push(tape::Delta {
            id: finish,
            before: finish_square,
            after: finish_square | solve::FINISH_BIT,
            burst: 1,
        });
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter_history(
                monitor_clone,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_start,
                    speed: 0,
                },
            );
        }));
    }
    hunter_history(
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

    if let Ok(mut lk) = monitor.lock() {
        let burst = 4;
        for i in (0..lk.maze.solve_history.len()).step_by(burst) {
            if i + burst <= lk.maze.solve_history.len() {
                lk.maze.solve_history[i].burst = burst;
                lk.maze.solve_history[i + burst - 1].burst = burst;
            }
        }
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            let square = lk.maze.get(p.0.row, p.0.col);
            lk.maze.solve_history.push(tape::Delta {
                id: p.0,
                before: square,
                after: (square & !solve::THREAD_MASK) | p.1,
                burst: 1,
            });
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

pub fn hunt(monitor: monitor::MazeReceiver) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        let start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                monitor_clone,
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

    if let Ok(mut lk) = monitor.solver.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            *lk.maze.get_mut(p.0.row, p.0.col) &= !solve::THREAD_MASK;
            *lk.maze.get_mut(p.0.row, p.0.col) |= p.1;
        }
        solve::print_paths(&lk.maze);
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

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
        let start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        solve::flush_cursor_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
        start
    } else {
        print::maze_panic!("Thread panic.");
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

    if let Ok(mut lk) = monitor.solver.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            *lk.maze.get_mut(p.0.row, p.0.col) &= !solve::THREAD_MASK;
            *lk.maze.get_mut(p.0.row, p.0.col) |= p.1;
            solve::flush_cursor_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

fn animate_mini_hunt(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        let start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        solve::flush_mini_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
        start
    } else {
        print::maze_panic!("Thread panic.");
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

    if let Ok(mut lk) = monitor.solver.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            *lk.maze.get_mut(p.0.row, p.0.col) &= !solve::THREAD_MASK;
            *lk.maze.get_mut(p.0.row, p.0.col) |= p.1;
            solve::flush_mini_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
}

pub fn gather_history(monitor: monitor::MazeMonitor) {
    todo!();
}

pub fn gather(monitor: monitor::MazeReceiver) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        let start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        }
        start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            gatherer(
                monitor_clone,
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

    if let Ok(lk) = monitor.solver.lock() {
        solve::print_paths(&lk.maze);
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
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
        let start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
            solve::flush_cursor_path_coordinate(&lk.maze, finish);
            thread::sleep(time::Duration::from_micros(animation));
        }
        start
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
}

fn animate_mini_gather(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let all_start: maze::Point = if let Ok(mut lk) = monitor.solver.lock() {
        let start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
            solve::flush_mini_path_coordinate(&lk.maze, finish);
            thread::sleep(time::Duration::from_micros(animation));
        }
        start
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
}

pub fn corner_history(monitor: monitor::MazeMonitor) {
    todo!();
}

pub fn corner(monitor: monitor::MazeReceiver) {
    let mut all_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.solver.lock() {
        let all_starts = solve::set_corner_starts(&lk.maze);
        for s in all_starts {
            *lk.maze.get_mut(s.row, s.col) |= solve::START_BIT;
        }
        let finish = maze::Point {
            row: lk.maze.rows() / 2,
            col: lk.maze.cols() / 2,
        };
        for p in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + p.row,
                col: finish.col + p.col,
            };
            *lk.maze.get_mut(next.row, next.col) |= maze::PATH_BIT;
        }
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        *lk.maze.get_mut(finish.row, finish.col) |= maze::PATH_BIT;
        all_starts
    } else {
        print::maze_panic!("Thread panick.");
    };

    all_starts.shuffle(&mut thread_rng());
    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                monitor_clone,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    start: all_starts[i_thread + 1],
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
            start: all_starts[0],
            speed: 0,
        },
    );

    for handle in handles {
        handle.join().unwrap();
    }

    if let Ok(mut lk) = monitor.solver.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            *lk.maze.get_mut(p.0.row, p.0.col) &= !solve::THREAD_MASK;
            *lk.maze.get_mut(p.0.row, p.0.col) |= p.1;
        }
        solve::print_paths(&lk.maze);
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
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
    let mut all_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.solver.lock() {
        let all_starts = solve::set_corner_starts(&lk.maze);
        for s in all_starts {
            *lk.maze.get_mut(s.row, s.col) |= solve::START_BIT;
            solve::flush_cursor_path_coordinate(&lk.maze, s);
            thread::sleep(time::Duration::from_micros(animation));
        }
        let finish = maze::Point {
            row: lk.maze.rows() / 2,
            col: lk.maze.cols() / 2,
        };
        for p in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + p.row,
                col: finish.col + p.col,
            };
            *lk.maze.get_mut(next.row, next.col) |= maze::PATH_BIT;
            solve::flush_cursor_path_coordinate(&lk.maze, next);
            thread::sleep(time::Duration::from_micros(animation));
        }
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        *lk.maze.get_mut(finish.row, finish.col) |= maze::PATH_BIT;
        solve::flush_cursor_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
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

    if let Ok(mut lk) = monitor.solver.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            *lk.maze.get_mut(p.0.row, p.0.col) &= !solve::THREAD_MASK;
            *lk.maze.get_mut(p.0.row, p.0.col) |= p.1;
            solve::flush_cursor_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
}

fn animate_mini_corner(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    let mut all_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.solver.lock() {
        let all_starts = solve::set_corner_starts(&lk.maze);
        for s in all_starts {
            *lk.maze.get_mut(s.row, s.col) |= solve::START_BIT;
            solve::flush_mini_path_coordinate(&lk.maze, s);
            thread::sleep(time::Duration::from_micros(animation));
        }
        let finish = maze::Point {
            row: lk.maze.rows() / 2,
            col: lk.maze.cols() / 2,
        };
        for p in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + p.row,
                col: finish.col + p.col,
            };
            *lk.maze.get_mut(next.row, next.col) |= maze::PATH_BIT;
            solve::flush_mini_path_coordinate(&lk.maze, next);
            thread::sleep(time::Duration::from_micros(animation));
        }
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        *lk.maze.get_mut(finish.row, finish.col) |= maze::PATH_BIT;
        solve::flush_mini_path_coordinate(&lk.maze, finish);
        thread::sleep(time::Duration::from_micros(animation));
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

    if let Ok(mut lk) = monitor.solver.lock() {
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            *lk.maze.get_mut(p.0.row, p.0.col) &= !solve::THREAD_MASK;
            *lk.maze.get_mut(p.0.row, p.0.col) |= p.1;
            solve::flush_mini_path_coordinate(&lk.maze, p.0);
            thread::sleep(time::Duration::from_micros(animation));
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn hunter_history(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.win.is_some() {
                return;
            }
            let square = lk.maze.get(cur.row, cur.col);
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                lk.maze.solve_history.push(tape::Delta {
                    id: cur,
                    before: square,
                    after: square | guide.paint,
                    burst: 1,
                });
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
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
            lk.maze.solve_history.push(tape::Delta {
                id: cur,
                before: square,
                after: square | guide.paint,
                burst: 1,
            });
            *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
        } else {
            print::maze_panic!("Thread panicked!");
        }

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

fn hunter(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.solver.lock() {
            if lk.win.is_some() {
                return;
            }
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
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
            *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
        } else {
            print::maze_panic!("Thread panicked!");
        }

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

fn animated_hunter(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(mut cur) = bfs.pop_front() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            if lk.win.is_some() {
                return;
            }
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
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
            *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
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
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

fn animated_mini_hunter(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(mut cur) = bfs.pop_front() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            if lk.win.is_some() {
                return;
            }
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
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
            *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
            solve::flush_mini_path_coordinate(&lk.maze, cur);
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
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

fn gatherer(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.solver.lock() {
            let finish = (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0;
            let first = (lk.maze.get(cur.row, cur.col) & solve::CACHE_MASK) == 0;
            // We can only stop looking if we are the first to this finish. Keep looking otherwise.
            match (finish, first) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen_bit;
                    return;
                }
                (true, false) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen_bit;
                }
                (_, _) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen_bit | guide.paint;
                }
            }
        } else {
            print::maze_panic!("Thread panicked!");
        }

        let mut i = guide.index;
        for _ in 0..solve::NUM_DIRECTIONS {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

fn animated_gatherer(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            let finish = (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0;
            let first = (lk.maze.get(cur.row, cur.col) & solve::CACHE_MASK) == 0;
            // We can only stop looking if we are the first to this finish. Keep looking otherwise.
            match (finish, first) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen_bit;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                    return;
                }
                (true, false) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen_bit;
                }
                (_, _) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen_bit | guide.paint;
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
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

fn animated_mini_gatherer(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let seen_bit: solve::ThreadCache = guide.paint << 4;
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if monitor.exit() {
            return;
        }
        if let Ok(mut lk) = monitor.solver.lock() {
            let finish = (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0;
            let first = (lk.maze.get(cur.row, cur.col) & solve::CACHE_MASK) == 0;
            // We can only stop looking if we are the first to this finish. Keep looking otherwise.
            match (finish, first) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | seen_bit;
                    solve::flush_mini_path_coordinate(&lk.maze, cur);
                    return;
                }
                (true, false) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen_bit;
                }
                (_, _) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= seen_bit | guide.paint;
                    solve::flush_mini_path_coordinate(&lk.maze, cur);
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
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}
