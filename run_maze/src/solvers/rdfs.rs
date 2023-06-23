use crate::maze;
use crate::utilities::print;
use crate::utilities::solve;

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
    for i_thread in 0..solve::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: solve::THREAD_MASKS[i_thread],
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
            solve::print_overlap_key();
            solve::print_hunt_solution_message(print_lock.win);
            println!();
        }
        Err(poison) => println!("Solve thread panic! somehow: {:?}", poison),
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
    for i_thread in 0..solve::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            gatherer(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: solve::THREAD_MASKS[i_thread],
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
            solve::print_overlap_key();
            solve::print_gather_solution_message();
            println!();
        }
        Err(poison) => println!("Solve thread panic! somehow: {:?}", poison),
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
    for i_thread in 0..solve::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: solve::THREAD_MASKS[i_thread],
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
            solve::print_overlap_key();
            solve::print_hunt_solution_message(print_lock.win);
            println!();
        }
        Err(poison) => println!("Solve thread panic!: {:?}", poison),
    };
}

pub fn animate_hunt(mut maze: maze::BoxMaze, speed: solve::SolverSpeed) {
    print::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve::print_overlap_key();
    let all_start: maze::Point = solve::pick_random_point(&maze);
    maze[all_start.row as usize][all_start.col as usize] |= solve::START_BIT;
    let finish: maze::Point = solve::pick_random_point(&maze);
    maze[finish.row as usize][finish.col as usize] |= solve::FINISH_BIT;
    let animation = solve::SOLVER_SPEEDS[speed as usize];
    solve::flush_cursor_path_coordinate(&maze, finish);
    thread::sleep(time::Duration::from_micros(animation));

    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for i_thread in 0..solve::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: solve::THREAD_MASKS[i_thread],
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
        Ok(print_lock) => {
            print::set_cursor_position(maze::Point {
                row: print_lock.maze.row_size() + solve::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
                col: 0,
            });
            solve::print_hunt_solution_message(print_lock.win);
            println!();
        }
        Err(poison) => println!("Solve thread panic!: {:?}", poison),
    };
}

pub fn animate_gather(mut maze: maze::BoxMaze, speed: solve::SolverSpeed) {
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

    let monitor: solve::SolverMonitor = solve::Solver::new(maze);
    let mut handles = Vec::with_capacity(solve::NUM_THREADS);
    for i_thread in 0..solve::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_gatherer(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: solve::THREAD_MASKS[i_thread],
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
        Ok(print_lock) => {
            print::set_cursor_position(maze::Point {
                row: print_lock.maze.row_size() + solve::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
                col: 0,
            });
            solve::print_gather_solution_message();
            println!();
        }
        Err(poison) => println!("Solve thread panic!: {:?}", poison),
    };
}

pub fn animate_corner(mut maze: maze::BoxMaze, speed: solve::SolverSpeed) {
    print::set_cursor_position(maze::Point {
        row: maze.row_size(),
        col: 0,
    });
    solve::print_overlap_key();
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
    for i_thread in 0..solve::NUM_THREADS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            animated_hunter(
                &mut monitor_clone,
                solve::ThreadGuide {
                    index: i_thread,
                    paint: solve::THREAD_MASKS[i_thread],
                    start: corner_starts[i_thread],
                    speed: animation,
                },
            );
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    match monitor.lock() {
        Ok(print_lock) => {
            print::set_cursor_position(maze::Point {
                row: print_lock.maze.row_size() + solve::OVERLAP_KEY_AND_MESSAGE_HEIGHT,
                col: 0,
            });
            solve::print_hunt_solution_message(print_lock.win);
            println!();
        }
        Err(poison) => println!("Solve thread panic!: {:?}", poison),
    };
}

// Dispatch Functions for each Thread--------------------------------------------------------------

fn hunter(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);

    let mut rng = thread_rng();
    let mut rng_arr: Vec<usize> = (0..solve::NUM_DIRECTIONS).collect();
    while let Some(&cur) = dfs.last() {
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
                        lk.win.get_or_insert(guide.index);
                        dfs.pop();
                        for p in dfs {
                            lk.maze[p.row as usize][p.col as usize] |= guide.paint;
                        }
                        return;
                    }
                    lk.maze[cur.row as usize][cur.col as usize] |= seen;
                }
            },
            Err(poison) => println!("Solve thread panic!: {:?}", poison),
        };

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut found_branch = false;
        rng_arr.shuffle(&mut rng);
        for &i in &rng_arr {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            let mut push_next = false;
            match monitor.lock() {
                Ok(lk) => {
                    push_next = (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
                Err(poison) => println!("Solve thread panic!: {:?}", poison),
            };

            if push_next {
                found_branch = true;
                dfs.push(next);
                break;
            }
        }

        if !found_branch {
            dfs.pop();
        }
    }
}

fn animated_hunter(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    let mut rng = thread_rng();
    let mut rng_arr: Vec<usize> = (0..solve::NUM_DIRECTIONS).collect();
    while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => match lk.win {
                Some(_) => return,
                None => {
                    if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0 {
                        lk.win.get_or_insert(guide.index);
                        dfs.pop();
                        return;
                    }
                    lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                }
            },
            Err(poison) => println!("Solve thread panic!: {:?}", poison),
        };

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut found_branch = false;
        rng_arr.shuffle(&mut rng);
        for &i in &rng_arr {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            let mut push_next = false;

            match monitor.lock() {
                Ok(lk) => {
                    push_next = (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0;
                }
                Err(poison) => println!("Solve thread panic!: {:?}", poison),
            }

            if push_next {
                found_branch = true;
                dfs.push(next);
                break;
            }
        }

        if !found_branch {
            match monitor.lock() {
                Ok(mut lk) => {
                    lk.maze[cur.row as usize][cur.col as usize] &= !guide.paint;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                }
                Err(poison) => println!("Solve thread panic!: {:?}", poison),
            }
            thread::sleep(time::Duration::from_micros(guide.speed));
            dfs.pop();
        }
    }
}

fn gatherer(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    let mut rng = thread_rng();
    let mut rng_arr: Vec<usize> = (0..solve::NUM_DIRECTIONS).collect();
    while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => {
                if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0
                    && (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0
                {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen;
                    dfs.pop();
                    for p in dfs {
                        lk.maze[p.row as usize][p.col as usize] |= guide.paint;
                    }
                    return;
                }
                lk.maze[cur.row as usize][cur.col as usize] |= seen;
            }
            Err(poison) => println!("Solve thread panic!: {:?}", poison),
        };

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut found_branch = false;
        rng_arr.shuffle(&mut rng);
        for &i in &rng_arr {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            let mut push_next = false;

            match monitor.lock() {
                Ok(lk) => {
                    push_next = (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0;
                }
                Err(poison) => {
                    println!("Solve thread panic!: {:?}", poison);
                }
            };

            if push_next {
                found_branch = true;
                dfs.push(next);
                break;
            }
        }

        if !found_branch {
            dfs.pop();
        }
    }
}

fn animated_gatherer(monitor: &mut solve::SolverMonitor, guide: solve::ThreadGuide) {
    let seen: solve::ThreadCache = guide.paint << solve::THREAD_TAG_OFFSET;
    let mut dfs: Vec<maze::Point> = Vec::with_capacity(solve::INITIAL_PATH_LEN);
    dfs.push(guide.start);
    let mut rng = thread_rng();
    let mut rng_arr: Vec<usize> = (0..solve::NUM_DIRECTIONS).collect();
    while let Some(&cur) = dfs.last() {
        match monitor.lock() {
            Ok(mut lk) => {
                if (lk.maze[cur.row as usize][cur.col as usize] & solve::FINISH_BIT) != 0
                    && (lk.maze[cur.row as usize][cur.col as usize] & solve::CACHE_MASK) == 0
                {
                    lk.maze[cur.row as usize][cur.col as usize] |= seen;
                    dfs.pop();
                    return;
                }
                lk.maze[cur.row as usize][cur.col as usize] |= seen | guide.paint;
                solve::flush_cursor_path_coordinate(&lk.maze, cur);
            }
            Err(poison) => println!("Solve thread panic!: {:?}", poison),
        }

        thread::sleep(time::Duration::from_micros(guide.speed));

        // Bias threads towards their original dispatch direction with do-while loop.
        let mut found_branch = false;
        rng_arr.shuffle(&mut rng);
        for &i in &rng_arr {
            let p: &maze::Point = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };

            let mut push_next = false;

            match monitor.lock() {
                Ok(lk) => {
                    push_next = (lk.maze[next.row as usize][next.col as usize] & seen) == 0
                        && (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0;
                }
                Err(poison) => println!("Solve thread panic!: {:?}", poison),
            };
            if push_next {
                found_branch = true;
                dfs.push(next);
                break;
            }
        }

        if !found_branch {
            match monitor.lock() {
                Ok(mut lk) => {
                    lk.maze[cur.row as usize][cur.col as usize] &= !guide.paint;
                    solve::flush_cursor_path_coordinate(&lk.maze, cur);
                }
                Err(poison) => println!("Solve thread panic!: {:?}", poison),
            }
            thread::sleep(time::Duration::from_micros(guide.speed));
            dfs.pop();
        }
    }
}
