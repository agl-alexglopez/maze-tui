use crate::solve;
use maze;
use print;

use rand::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::thread;

const BURST: usize = 4;

///
/// Data only solvers------------------------------------------------------------------------------
///
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
                    cache: solve::THREAD_CACHES[i_thread + 1],
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
            cache: solve::THREAD_CACHES[0],
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
        return;
    }
    print::maze_panic!("Thread panicked with the lock!");
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
            *lk.maze.get_mut(next.row, next.col) =
                (lk.maze.get(next.row, next.col) & !maze::WALL_MASK) | maze::PATH_BIT;
        }
        *lk.maze.get_mut(finish.row, finish.col) = (lk.maze.get(finish.row, finish.col)
            & !maze::WALL_MASK)
            | solve::FINISH_BIT
            | maze::PATH_BIT;
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
                    cache: solve::THREAD_CACHES[i_thread + 1],
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
            cache: solve::THREAD_CACHES[0],
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
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
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
                Ok(lk) => lk.maze.path_at(next.row, next.col),
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
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
                    cache: solve::THREAD_CACHES[i_thread + 1],
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
            cache: solve::THREAD_CACHES[0],
            start: all_start,
            speed: 0,
        },
    );

    for handle in handles {
        handle.join().unwrap();
    }
}

fn gatherer(monitor: monitor::MazeReceiver, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.solver.lock() {
            let square = lk.maze.get(cur.row, cur.col);
            // We can only stop looking if we are the first to this finish. Keep looking otherwise.
            match (solve::is_finish(square), solve::is_first(square)) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | guide.cache;
                    return;
                }
                (true, false) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.cache;
                }
                (_, _) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.cache | guide.paint;
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
                Ok(lk) => lk.maze.path_at(next.row, next.col),
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

///
/// History based solvers for recording and playback-----------------------------------------------
///
pub fn hunt_history(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let start = solve::pick_random_point(&lk.maze);
        let start_square = lk.maze.get(start.row, start.col);
        lk.maze.solve_history.push(maze::Delta {
            id: start,
            before: start_square,
            after: start_square | solve::START_BIT,
            burst: BURST,
        });
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        let finish_square = lk.maze.get(finish.row, finish.col);
        lk.maze.solve_history.push(maze::Delta {
            id: finish,
            before: finish_square,
            after: finish_square | solve::FINISH_BIT,
            burst: BURST,
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
                    cache: solve::THREAD_CACHES[i_thread + 1],
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
            cache: solve::THREAD_CACHES[0],
            start: all_start,
            speed: 0,
        },
    );

    for handle in handles {
        handle.join().unwrap();
    }

    if let Ok(mut lk) = monitor.lock() {
        // I kind of cheated by having every history claim it was a 4-burst. That works but we need
        // to tidy up so when we start reversing from the end the jumps by 4-bursts are correct.
        let len = lk.maze.solve_history.len();
        if len % BURST != 0 {
            lk.maze
                .solve_history
                .slice_mut(len - (len % BURST), len)
                .iter_mut()
                .for_each(|s| s.burst = 1);
        }
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            let square = lk.maze.get(p.0.row, p.0.col);
            lk.maze.solve_history.push(maze::Delta {
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

pub fn corner_history(monitor: monitor::MazeMonitor) {
    let mut all_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        let all_starts = solve::set_corner_starts(&lk.maze);
        for s in all_starts {
            let start_square = lk.maze.get(s.row, s.col);
            lk.maze.solve_history.push(maze::Delta {
                id: s,
                before: start_square,
                after: start_square | solve::START_BIT,
                burst: BURST,
            });
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
            let next_square = lk.maze.get(next.row, next.col);
            lk.maze.solve_history.push(maze::Delta {
                id: next,
                before: next_square,
                after: (next_square & !maze::WALL_MASK) | maze::PATH_BIT,
                burst: BURST,
            });
            *lk.maze.get_mut(next.row, next.col) =
                (next_square & !maze::WALL_MASK) | maze::PATH_BIT;
        }
        let finish_square = lk.maze.get(finish.row, finish.col);
        lk.maze.solve_history.push(maze::Delta {
            id: finish,
            before: finish_square,
            after: (finish_square & !maze::WALL_MASK) | solve::FINISH_BIT | maze::PATH_BIT,
            burst: BURST,
        });
        *lk.maze.get_mut(finish.row, finish.col) =
            (finish_square & !maze::WALL_MASK) | solve::FINISH_BIT | maze::PATH_BIT;
        all_starts
    } else {
        print::maze_panic!("Thread panick.");
    };

    all_starts.shuffle(&mut thread_rng());
    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter_history(
                monitor_clone,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    cache: solve::THREAD_CACHES[i_thread + 1],
                    start: all_starts[i_thread + 1],
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
            cache: solve::THREAD_CACHES[0],
            start: all_starts[0],
            speed: 0,
        },
    );

    for handle in handles {
        handle.join().unwrap();
    }

    if let Ok(mut lk) = monitor.lock() {
        let len = lk.maze.solve_history.len();
        if len % BURST != 0 {
            lk.maze
                .solve_history
                .slice_mut(len - (len % BURST) + 1, len)
                .iter_mut()
                .for_each(|s| s.burst = 1);
        }
        for i in 0..lk.win_path.len() {
            let p = lk.win_path[i];
            let square = lk.maze.get(p.0.row, p.0.col);
            lk.maze.solve_history.push(maze::Delta {
                id: p.0,
                before: square,
                after: (square & !solve::THREAD_MASK) | p.1,
                burst: 1,
            });
            *lk.maze.get_mut(p.0.row, p.0.col) = (square & !solve::THREAD_MASK) | p.1;
        }
        return;
    }
    print::maze_panic!("Thread panicked with the lock");
}

fn hunter_history(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.win.is_some() {
                return;
            }
            let square = lk.maze.get(cur.row, cur.col);
            if solve::is_finish(lk.maze.get(cur.row, cur.col)) {
                lk.maze.solve_history.push(maze::Delta {
                    id: cur,
                    before: square,
                    after: square | guide.paint,
                    burst: BURST,
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
            lk.maze.solve_history.push(maze::Delta {
                id: cur,
                before: square,
                after: square | guide.paint,
                burst: BURST,
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
                Ok(lk) => maze::is_path(lk.maze.get(next.row, next.col)),
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}

pub fn gather_history(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let start = solve::pick_random_point(&lk.maze);
        let start_square = lk.maze.get(start.row, start.col);
        lk.maze.solve_history.push(maze::Delta {
            id: start,
            before: start_square,
            after: start_square | solve::START_BIT,
            burst: BURST,
        });
        *lk.maze.get_mut(start.row, start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            let finish_square = lk.maze.get(finish.row, finish.col);
            lk.maze.solve_history.push(maze::Delta {
                id: finish,
                before: finish_square,
                after: finish_square | solve::FINISH_BIT,
                burst: BURST,
            });
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        }
        start
    } else {
        print::maze_panic!("Thread panick.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            gatherer_history(
                monitor_clone,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    cache: solve::THREAD_CACHES[i_thread + 1],
                    start: all_start,
                    speed: 0,
                },
            );
        }));
    }
    gatherer_history(
        monitor.clone(),
        solve::ThreadGuide {
            index: 0,
            paint: solve::THREAD_MASKS[0],
            cache: solve::THREAD_CACHES[0],
            start: all_start,
            speed: 0,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
    if let Ok(mut lk) = monitor.lock() {
        let len = lk.maze.solve_history.len();
        if len % BURST != 0 {
            lk.maze
                .solve_history
                .slice_mut(len - (len % BURST), len)
                .iter_mut()
                .for_each(|s| s.burst = 1);
        }
        return;
    }
    print::maze_panic!("thread panick.");
}

fn gatherer_history(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut parents = HashMap::from([(guide.start, maze::Point { row: -1, col: -1 })]);
    let mut bfs: VecDeque<maze::Point> = VecDeque::from([guide.start]);
    while let Some(cur) = bfs.pop_front() {
        if let Ok(mut lk) = monitor.lock() {
            let before = lk.maze.get(cur.row, cur.col);
            // We can only stop looking if we are the first to this finish. Keep looking otherwise.
            match (solve::is_finish(before), solve::is_first(before)) {
                (true, true) => {
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before,
                        after: before | guide.paint | guide.cache,
                        burst: BURST,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | guide.cache;
                    return;
                }
                (true, false) => {
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before,
                        after: before | guide.cache,
                        burst: BURST,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.cache;
                }
                _ => {
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before,
                        after: before | guide.cache | guide.paint,
                        burst: BURST,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.cache | guide.paint;
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
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Thread panicked: {}", p),
                Ok(lk) => maze::is_path(lk.maze.get(next.row, next.col)),
            } && !parents.contains_key(&next)
            {
                parents.insert(next, cur);
                bfs.push_back(next);
            }
            i = (i + 1) % solve::NUM_DIRECTIONS;
        }
    }
}
