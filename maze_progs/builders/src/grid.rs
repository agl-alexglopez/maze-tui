use crate::build;
use maze;
use speed;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{thread, time};

const RUN_LIMIT: i32 = 4;

struct RunStart {
    cur: maze::Point,
    dir: maze::Point,
}

///
/// Data only maze generator
///

pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    let mut rng = thread_rng();
    let mut dfs: Vec<maze::Point> = Vec::from([maze::Point {
        row: 2 * (rng.gen_range(1..lk.maze.rows() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(1..lk.maze.cols() - 1) / 2) + 1,
    }]);
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    while let Some(run) = dfs.last().cloned() {
        random_direction_indices.shuffle(&mut rng);
        let mut branches = false;
        for &i in random_direction_indices.iter() {
            let p = build::GENERATE_DIRECTIONS[i];
            let next = maze::Point {
                row: run.row + p.row,
                col: run.col + p.col,
            };
            if build::can_build_new_square(&lk.maze, next) {
                complete_run(&mut lk.maze, &mut dfs, RunStart { cur: run, dir: p });
                branches = true;
                break;
            }
        }
        if !branches {
            dfs.pop();
        }
    }
}

fn complete_run(maze: &mut maze::Maze, dfs: &mut Vec<maze::Point>, mut run: RunStart) {
    let mut next = maze::Point {
        row: run.cur.row + run.dir.row,
        col: run.cur.col + run.dir.col,
    };
    let mut cur_run = 0;
    while build::is_square_within_perimeter_walls(maze, next) && cur_run < RUN_LIMIT {
        build::join_squares(maze, run.cur, next);
        dfs.push(next);
        run.cur = next;
        next.row += run.dir.row;
        next.col += run.dir.col;
        cur_run += 1;
    }
}

///
/// History based generator for animation and playback.
///

pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_history_with_walls(&mut lk.maze);
    let mut rng = thread_rng();
    let mut dfs: Vec<maze::Point> = Vec::from([maze::Point {
        row: 2 * (rng.gen_range(1..lk.maze.rows() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(1..lk.maze.cols() - 1) / 2) + 1,
    }]);
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    while let Some(run) = dfs.last().cloned() {
        random_direction_indices.shuffle(&mut rng);
        let mut branches = false;
        for &i in random_direction_indices.iter() {
            let p = build::GENERATE_DIRECTIONS[i];
            let next = maze::Point {
                row: run.row + p.row,
                col: run.col + p.col,
            };
            if build::can_build_new_square(&lk.maze, next) {
                complete_run_history(&mut lk.maze, &mut dfs, RunStart { cur: run, dir: p });
                branches = true;
                break;
            }
        }
        if !branches {
            dfs.pop();
        }
    }
}

fn complete_run_history(maze: &mut maze::Maze, dfs: &mut Vec<maze::Point>, mut run: RunStart) {
    let mut next = maze::Point {
        row: run.cur.row + run.dir.row,
        col: run.cur.col + run.dir.col,
    };
    let mut cur_run = 0;
    while build::is_square_within_perimeter_walls(maze, next) && cur_run < RUN_LIMIT {
        build::join_squares_history(maze, run.cur, next);
        dfs.push(next);
        run.cur = next;
        next.row += run.dir.row;
        next.col += run.dir.col;
        cur_run += 1;
    }
}

///
/// Cursor based generator
///

pub fn animate_maze(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    if lk.maze.is_mini() {
        drop(lk);
        animate_mini_maze(monitor, speed);
        return;
    }
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
        let mut rng = thread_rng();
    let mut dfs: Vec<maze::Point> = Vec::from([maze::Point {
        row: 2 * (rng.gen_range(1..lk.maze.rows() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(1..lk.maze.cols() - 1) / 2) + 1,
    }]);
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    while let Some(run) = dfs.last().cloned() {
        if monitor.exit() {
            return;
        }
        random_direction_indices.shuffle(&mut rng);
        let mut branches = false;
        for &i in random_direction_indices.iter() {
            let p = build::GENERATE_DIRECTIONS[i];
            let next = maze::Point {
                row: run.row + p.row,
                col: run.col + p.col,
            };
            if build::can_build_new_square(&lk.maze, next) {
                animate_run(
                    &mut lk.maze,
                    &mut dfs,
                    RunStart { cur: run, dir: p },
                    animation,
                );
                branches = true;
                break;
            }
        }
        if !branches {
            build::flush_cursor_maze_coordinate(&lk.maze, run);
            thread::sleep(time::Duration::from_micros(animation));
            dfs.pop();
        }
    }
}

fn animate_mini_maze(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
        let mut rng = thread_rng();
    let mut dfs: Vec<maze::Point> = Vec::from([maze::Point {
        row: 2 * (rng.gen_range(1..lk.maze.rows() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(1..lk.maze.cols() - 1) / 2) + 1,
    }]);
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    while let Some(run) = dfs.last().cloned() {
        if monitor.exit() {
            return;
        }
        random_direction_indices.shuffle(&mut rng);
        let mut branches = false;
        for &i in random_direction_indices.iter() {
            let p = build::GENERATE_DIRECTIONS[i];
            let next = maze::Point {
                row: run.row + p.row,
                col: run.col + p.col,
            };
            if build::can_build_new_square(&lk.maze, next) {
                animate_mini_run(
                    &mut lk.maze,
                    &mut dfs,
                    RunStart { cur: run, dir: p },
                    animation,
                );
                branches = true;
                break;
            }
        }
        if !branches {
            build::flush_mini_coordinate(&lk.maze, run);
            thread::sleep(time::Duration::from_micros(animation));
            dfs.pop();
        }
    }
}

fn animate_run(
    maze: &mut maze::Maze,
    dfs: &mut Vec<maze::Point>,
    mut run: RunStart,
    animation: build::SpeedUnit,
) {
    let mut next = maze::Point {
        row: run.cur.row + run.dir.row,
        col: run.cur.col + run.dir.col,
    };
    let mut cur_run = 0;
    while build::is_square_within_perimeter_walls(maze, next) && cur_run < RUN_LIMIT {
        build::join_squares_animated(maze, run.cur, next, animation);
        dfs.push(next);
        run.cur = next;
        next.row += run.dir.row;
        next.col += run.dir.col;
        cur_run += 1;
    }
}

fn animate_mini_run(
    maze: &mut maze::Maze,
    dfs: &mut Vec<maze::Point>,
    mut run: RunStart,
    animation: build::SpeedUnit,
) {
    let mut next = maze::Point {
        row: run.cur.row + run.dir.row,
        col: run.cur.col + run.dir.col,
    };
    let mut cur_run = 0;
    while build::is_square_within_perimeter_walls(maze, next) && cur_run < RUN_LIMIT {
        build::join_minis_animated(maze, run.cur, next, animation);
        dfs.push(next);
        run.cur = next;
        next.row += run.dir.row;
        next.col += run.dir.col;
        cur_run += 1;
    }
}
