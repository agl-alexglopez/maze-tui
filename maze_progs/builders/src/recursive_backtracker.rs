use crate::build;
use maze;
use speed;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{thread, time};

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    let mut cur: maze::Point = start;
    let mut branches_remain: bool = true;
    while branches_remain {
        random_direction_indices.shuffle(&mut gen);
        branches_remain = false;
        for i in &random_direction_indices {
            let direction: &maze::Point = &build::GENERATE_DIRECTIONS[*i];
            let next = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(maze, next) {
                branches_remain = true;
                build::carve_path_markings(maze, cur, next);
                cur = next;
                break;
            }
        }
        if !branches_remain && cur != start {
            let dir: build::BacktrackMarker = (maze[cur.row as usize][cur.col as usize]
                & build::MARKERS_MASK)
                >> build::MARKER_SHIFT;
            // The solvers will need these bits later so we need to clear bits.
            maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
            let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
            cur.row += backtracking.row;
            cur.col += backtracking.col;
            branches_remain = true;
        }
    }
}

pub fn animate_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls_animated(maze);
    build::clear_and_flush_grid(maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..maze.row_size() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..maze.col_size() - 2) / 2) + 1,
    };
    let mut random_direction_indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    let mut cur: maze::Point = start;
    let mut branches_remain: bool = true;
    while branches_remain {
        random_direction_indices.shuffle(&mut gen);
        branches_remain = false;
        for i in &random_direction_indices {
            let direction = &build::GENERATE_DIRECTIONS[*i];
            let next = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(maze, next) {
                branches_remain = true;
                build::carve_path_markings_animated(maze, cur, next, animation);
                cur = next;
                break;
            }
        }
        if !branches_remain && cur != start {
            let dir: build::BacktrackMarker = (maze[cur.row as usize][cur.col as usize]
                & build::MARKERS_MASK)
                >> build::MARKER_SHIFT;
            // The solvers will need these bits later so we need to clear bits.
            maze[cur.row as usize][cur.col as usize] &= !build::MARKERS_MASK;
            let backtracking: &maze::Point = &build::BACKTRACKING_POINTS[dir as usize];
            build::flush_cursor_maze_coordinate(maze, cur);
            thread::sleep(time::Duration::from_micros(animation));
            cur.row += backtracking.row;
            cur.col += backtracking.col;
            branches_remain = true;
        }
    }
}
