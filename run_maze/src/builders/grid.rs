use crate::build;
use crate::maze;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{thread, time};

const RUN_LIMIT: i32 = 4;

struct RunStart {
    cur: maze::Point,
    dir: maze::Point,
}

// Public Functions-------------------------------------------------------------------------------

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls_animated(maze);
    let mut rng = thread_rng();
    let mut dfs: Vec<maze::Point> = Vec::from([maze::Point {
        row: 2 * (rng.gen_range(1..maze.row_size() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(1..maze.col_size() - 1) / 2) + 1,
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
            if build::can_build_new_square(maze, next) {
                complete_run(maze, &mut dfs, RunStart { cur: run, dir: p });
                branches = true;
                break;
            }
        }
        if !branches {
            dfs.pop();
        }
    }
    build::clear_and_flush_grid(maze);
}

pub fn animate_maze(maze: &mut maze::Maze, speed: build::BuilderSpeed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls_animated(maze);
    build::clear_and_flush_grid(maze);
    let mut rng = thread_rng();
    let mut dfs: Vec<maze::Point> = Vec::from([maze::Point {
        row: 2 * (rng.gen_range(1..maze.row_size() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(1..maze.col_size() - 1) / 2) + 1,
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
            if build::can_build_new_square(maze, next) {
                animate_run(maze, &mut dfs, RunStart { cur: run, dir: p }, animation);
                branches = true;
                break;
            }
        }
        if !branches {
            build::flush_cursor_maze_coordinate(maze, run);
            thread::sleep(time::Duration::from_micros(animation));
            dfs.pop();
        }
    }
}

// Private Helper Functions-----------------------------------------------------------------------

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
