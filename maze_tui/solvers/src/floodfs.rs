use crate::solve;
use maze;
use print;

use rand::prelude::*;
use std::thread;

///
/// Data only solvers------------------------------------------------------------------------------
///
pub fn hunt(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let all_start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
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
}

pub fn corner(monitor: monitor::MazeMonitor) {
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            *lk.maze.get_mut(p.row, p.col) |= solve::START_BIT;
        }
        let finish = maze::Point {
            row: lk.maze.rows() / 2,
            col: lk.maze.cols() / 2,
        };
        for d in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + d.row,
                col: finish.col + d.col,
            };
            *lk.maze.get_mut(next.row, next.col) =
                (lk.maze.get(next.row, next.col) & !maze::WALL_MASK) | maze::PATH_BIT;
        }
        *lk.maze.get_mut(finish.row, finish.col) = (lk.maze.get(finish.row, finish.col)
            & !maze::WALL_MASK)
            | solve::FINISH_BIT
            | maze::PATH_BIT;
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
                    cache: solve::THREAD_CACHES[i_thread + 1],
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
            cache: solve::THREAD_CACHES[0],
            start: corner_starts[0],
            speed: 0,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

fn hunter(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);

    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            if lk.win.is_some() {
                for p in dfs {
                    *lk.maze.get_mut(p.row, p.col) |= guide.paint;
                }
                return;
            }
            if (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0 {
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
                lk.win.get_or_insert(guide.index);
                return;
            }
            *lk.maze.get_mut(cur.row, cur.col) |= guide.cache | guide.paint;
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        // Bias threads towards their original dispatch direction.
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
                    let square = lk.maze.get(next.row, next.col);
                    (square & guide.cache) == 0 && maze::is_path(square)
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

pub fn gather(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let all_start = solve::pick_random_point(&lk.maze);
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
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

fn gatherer(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            match (
                (lk.maze.get(cur.row, cur.col) & solve::FINISH_BIT) != 0,
                (lk.maze.get(cur.row, cur.col) & solve::CACHE_MASK) == 0,
            ) {
                (true, true) => {
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.cache | guide.paint;
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
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        // Bias threads towards their original dispatch direction.
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
                    let square = lk.maze.get(next.row, next.col);
                    (square & guide.cache) == 0 && maze::is_path(square)
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

///
/// History based solvers for recording and playback-----------------------------------------------
///
pub fn hunt_history(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let all_start = solve::pick_random_point(&lk.maze);
        let start_square = lk.maze.get(all_start.row, all_start.col);
        lk.maze.solve_history.push(maze::Delta {
            id: all_start,
            before: start_square,
            after: start_square | solve::START_BIT,
            burst: 1,
        });
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        let finish: maze::Point = solve::pick_random_point(&lk.maze);
        let finish_square = lk.maze.get(finish.row, finish.col);
        lk.maze.solve_history.push(maze::Delta {
            id: finish,
            before: finish_square,
            after: finish_square | solve::FINISH_BIT,
            burst: 1,
        });
        *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter_history(
                mc,
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
}

pub fn corner_history(monitor: monitor::MazeMonitor) {
    let mut corner_starts: [maze::Point; 4] = if let Ok(mut lk) = monitor.lock() {
        let corner_starts = solve::set_corner_starts(&lk.maze);
        for p in corner_starts {
            let start_square = lk.maze.get(p.row, p.col);
            lk.maze.solve_history.push(maze::Delta {
                id: p,
                before: start_square,
                after: start_square | solve::START_BIT,
                burst: 1,
            });
            *lk.maze.get_mut(p.row, p.col) |= solve::START_BIT;
        }

        let finish = maze::Point {
            row: lk.maze.rows() / 2,
            col: lk.maze.cols() / 2,
        };
        for d in maze::ALL_DIRECTIONS {
            let next = maze::Point {
                row: finish.row + d.row,
                col: finish.col + d.col,
            };
            let next_square = lk.maze.get(next.row, next.col);
            lk.maze.solve_history.push(maze::Delta {
                id: next,
                before: next_square,
                after: (next_square & !maze::WALL_MASK) | maze::PATH_BIT,
                burst: 1,
            });
            *lk.maze.get_mut(next.row, next.col) =
                (next_square & !maze::WALL_MASK) | maze::PATH_BIT;
        }
        let finish_square = lk.maze.get(finish.row, finish.col);
        lk.maze.solve_history.push(maze::Delta {
            id: finish,
            before: finish_square,
            after: (finish_square & !maze::WALL_MASK) | solve::FINISH_BIT | maze::PATH_BIT,
            burst: 1,
        });
        *lk.maze.get_mut(finish.row, finish.col) =
            (finish_square & !maze::WALL_MASK) | solve::FINISH_BIT | maze::PATH_BIT;
        corner_starts
    } else {
        print::maze_panic!("Thread panic.");
    };

    corner_starts.shuffle(&mut thread_rng());
    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter_history(
                mc,
                solve::ThreadGuide {
                    index: i_thread + 1,
                    paint: mask,
                    cache: solve::THREAD_CACHES[i_thread + 1],
                    start: corner_starts[i_thread + 1],
                    speed: 1,
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
            start: corner_starts[0],
            speed: 0,
        },
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

fn hunter_history(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
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
                    burst: 1,
                });
                *lk.maze.get_mut(cur.row, cur.col) |= guide.paint;
                lk.win.get_or_insert(guide.index);
                return;
            }
            lk.maze.solve_history.push(maze::Delta {
                id: cur,
                before: square,
                after: square | guide.cache | guide.paint,
                burst: 1,
            });
            *lk.maze.get_mut(cur.row, cur.col) |= guide.cache | guide.paint;
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        // Bias threads towards their original dispatch direction.
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
                    let square = lk.maze.get(next.row, next.col);
                    (square & guide.cache) == 0 && maze::is_path(square)
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

pub fn gather_history(monitor: monitor::MazeMonitor) {
    let all_start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let all_start = solve::pick_random_point(&lk.maze);
        let start_square = lk.maze.get(all_start.row, all_start.col);
        lk.maze.solve_history.push(maze::Delta {
            id: all_start,
            before: start_square,
            after: start_square | solve::START_BIT,
            burst: 1,
        });
        *lk.maze.get_mut(all_start.row, all_start.col) |= solve::START_BIT;
        for _ in 0..solve::NUM_GATHER_FINISHES {
            let finish: maze::Point = solve::pick_random_point(&lk.maze);
            let finish_square = lk.maze.get(finish.row, finish.col);
            lk.maze.solve_history.push(maze::Delta {
                id: finish,
                before: finish_square,
                after: finish_square | solve::FINISH_BIT,
                burst: 1,
            });
            *lk.maze.get_mut(finish.row, finish.col) |= solve::FINISH_BIT;
        }
        all_start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut handles = Vec::with_capacity(solve::NUM_THREADS - 1);
    for (i_thread, &mask) in solve::THREAD_MASKS.iter().skip(1).enumerate() {
        let mc = monitor.clone();
        handles.push(thread::spawn(move || {
            gatherer_history(
                mc,
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
}

fn gatherer_history(monitor: monitor::MazeMonitor, guide: solve::ThreadGuide) {
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    'branching: while let Some(&cur) = dfs.last() {
        if let Ok(mut lk) = monitor.lock() {
            let square = lk.maze.get(cur.row, cur.col);
            match (solve::is_finish(square), solve::is_first(square)) {
                (true, true) => {
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before: square,
                        after: square | guide.cache | guide.paint,
                        burst: 1,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | guide.cache;
                    return;
                }
                (true, false) => {
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before: square,
                        after: square | guide.cache,
                        burst: 1,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.cache;
                }
                (_, _) => {
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before: square,
                        after: square | guide.cache | guide.paint,
                        burst: 1,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= guide.paint | guide.cache;
                }
            }
        } else {
            print::maze_panic!("Solve thread print::maze_panic!");
        }

        // Bias threads towards their original dispatch direction.
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
                    let square = lk.maze.get(next.row, next.col);
                    (square & guide.cache) == 0 && maze::is_path(square)
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
