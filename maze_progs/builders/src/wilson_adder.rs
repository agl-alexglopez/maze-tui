use crate::build;
use maze;
use print;
use speed;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{thread, time};

const WALK_BIT: maze::Square = 0b0100_0000_0000_0000;

#[derive(Clone, Copy)]
struct Loop {
    walk: maze::Point,
    root: maze::Point,
}

#[derive(Clone, Copy)]
struct RandomWalk {
    // I scan for a random walk starts row by row. This way we don't check built rows.
    prev_row_start: i32,
    prev: maze::Point,
    walk: maze::Point,
    next: maze::Point,
}

// Public Functions-------------------------------------------------------------------------------

pub fn generate_maze(monitor: monitor::SolverReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::build_wall_outline(&mut lk.maze);
    let mut rng = thread_rng();
    let mut cur = RandomWalk {
        prev_row_start: 2,
        prev: maze::Point { row: 0, col: 0 },
        walk: maze::Point {
            row: 2 * (rng.gen_range(2..lk.maze.row_size() - 1) / 2),
            col: 2 * (rng.gen_range(2..lk.maze.col_size() - 1) / 2),
        },
        next: maze::Point { row: 0, col: 0 },
    };
    let mut indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    'walking: loop {
        *lk.maze.get_mut(cur.walk.row, cur.walk.col) |= WALK_BIT;
        indices.shuffle(&mut rng);
        'choosing_step: for &i in indices.iter() {
            let p = &build::GENERATE_DIRECTIONS[i];
            cur.next = maze::Point {
                row: cur.walk.row + p.row,
                col: cur.walk.col + p.col,
            };
            if !is_valid_step(&lk.maze, cur.next, cur.prev) {
                continue 'choosing_step;
            }
            match complete_walk(&mut lk.maze, cur) {
                Some(new_walk) => {
                    cur = new_walk;
                    continue 'walking;
                }
                None => {
                    return;
                }
            }
        }
    }
}

pub fn animate_maze(monitor: monitor::SolverReceiver, speed: speed::Speed) {
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
    build::build_wall_outline(&mut lk.maze);
    build::flush_grid(&lk.maze);
    build::print_overlap_key_animated(&lk.maze);
    let mut rng = thread_rng();
    let mut cur = RandomWalk {
        prev_row_start: 2,
        prev: maze::Point { row: 0, col: 0 },
        walk: maze::Point {
            row: 2 * (rng.gen_range(2..lk.maze.row_size() - 1) / 2),
            col: 2 * (rng.gen_range(2..lk.maze.col_size() - 1) / 2),
        },
        next: maze::Point { row: 0, col: 0 },
    };
    let mut indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    'walking: loop {
        if monitor.exit() {
            return;
        }
        *lk.maze.get_mut(cur.walk.row, cur.walk.col) |= WALK_BIT;
        indices.shuffle(&mut rng);
        'choosing_step: for &i in indices.iter() {
            let p = &build::GENERATE_DIRECTIONS[i];
            cur.next = maze::Point {
                row: cur.walk.row + p.row,
                col: cur.walk.col + p.col,
            };
            if !is_valid_step(&lk.maze, cur.next, cur.prev) {
                continue 'choosing_step;
            }

            match complete_walk_animated(&mut lk.maze, cur, animation) {
                Some(new_walk) => {
                    cur = new_walk;
                    continue 'walking;
                }
                None => {
                    return;
                }
            }
        }
    }
}

fn animate_mini_maze(monitor: monitor::SolverReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::build_wall_outline(&mut lk.maze);
    build::flush_grid(&lk.maze);
    build::print_overlap_key_animated(&lk.maze);
    let mut rng = thread_rng();
    let mut cur = RandomWalk {
        prev_row_start: 2,
        prev: maze::Point { row: 0, col: 0 },
        walk: maze::Point {
            row: 2 * (rng.gen_range(2..lk.maze.row_size() - 1) / 2),
            col: 2 * (rng.gen_range(2..lk.maze.col_size() - 1) / 2),
        },
        next: maze::Point { row: 0, col: 0 },
    };
    let mut indices: Vec<usize> = (0..build::NUM_DIRECTIONS).collect();
    'walking: loop {
        if monitor.exit() {
            return;
        }
        *lk.maze.get_mut(cur.walk.row, cur.walk.col) |= WALK_BIT;
        indices.shuffle(&mut rng);
        'choosing_step: for &i in indices.iter() {
            let p = &build::GENERATE_DIRECTIONS[i];
            cur.next = maze::Point {
                row: cur.walk.row + p.row,
                col: cur.walk.col + p.col,
            };
            if !is_valid_step(&lk.maze, cur.next, cur.prev) {
                continue 'choosing_step;
            }

            match complete_mini_walk_animated(&mut lk.maze, cur, animation) {
                Some(new_walk) => {
                    cur = new_walk;
                    continue 'walking;
                }
                None => {
                    return;
                }
            }
        }
    }
}

// Private Functions------------------------------------------------------------------------------

fn complete_walk(maze: &mut maze::Maze, mut walk: RandomWalk) -> Option<RandomWalk> {
    if build::has_builder_bit(maze, walk.next) {
        build_with_marks(maze, walk.walk, walk.next);
        connect_walk(maze, walk.walk);
        match build::choose_point_from_row_start(
            maze,
            walk.prev_row_start,
            build::ParityPoint::Even,
        ) {
            Some(point) => {
                walk.prev_row_start = point.row;
                walk.walk = point;
                *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
                walk.prev = maze::Point { row: 0, col: 0 };
                return Some(walk);
            }
            None => {
                return None;
            }
        };
    }
    if found_loop(maze, walk.next) {
        erase_loop(
            maze,
            Loop {
                walk: walk.walk,
                root: walk.next,
            },
        );
        walk.walk = walk.next;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        walk.prev = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        return Some(walk);
    }
    build::mark_origin(maze, walk.walk, walk.next);
    walk.prev = walk.walk;
    walk.walk = walk.next;
    Some(walk)
}

fn complete_walk_animated(
    maze: &mut maze::Maze,
    mut walk: RandomWalk,
    speed: build::SpeedUnit,
) -> Option<RandomWalk> {
    if build::has_builder_bit(maze, walk.next) {
        build_with_marks_animated(maze, walk.walk, walk.next, speed);
        connect_walk_animated(maze, walk.walk, speed);
        match build::choose_point_from_row_start(
            maze,
            walk.prev_row_start,
            build::ParityPoint::Even,
        ) {
            Some(point) => {
                walk.prev_row_start = point.row;
                walk.walk = point;
                *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
                walk.prev = maze::Point { row: 0, col: 0 };
                return Some(walk);
            }
            None => {
                return None;
            }
        };
    }
    if found_loop(maze, walk.next) {
        erase_loop_animated(
            maze,
            Loop {
                walk: walk.walk,
                root: walk.next,
            },
            speed,
        );
        walk.walk = walk.next;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        walk.prev = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        return Some(walk);
    }
    build::mark_origin_animated(maze, walk.walk, walk.next, speed);
    walk.prev = walk.walk;
    walk.walk = walk.next;
    Some(walk)
}

fn complete_mini_walk_animated(
    maze: &mut maze::Maze,
    mut walk: RandomWalk,
    speed: build::SpeedUnit,
) -> Option<RandomWalk> {
    if build::has_builder_bit(maze, walk.next) {
        build_with_mini_marks_animated(maze, walk.walk, walk.next, speed);
        connect_mini_walk_animated(maze, walk.walk, speed);
        match build::choose_point_from_row_start(
            maze,
            walk.prev_row_start,
            build::ParityPoint::Even,
        ) {
            Some(point) => {
                walk.prev_row_start = point.row;
                walk.walk = point;
                *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
                walk.prev = maze::Point { row: 0, col: 0 };
                return Some(walk);
            }
            None => {
                return None;
            }
        };
    }
    if found_loop(maze, walk.next) {
        erase_mini_loop_animated(
            maze,
            Loop {
                walk: walk.walk,
                root: walk.next,
            },
            speed,
        );
        walk.walk = walk.next;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        walk.prev = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        return Some(walk);
    }
    build::mark_mini_origin_animated(maze, walk.walk, walk.next, speed);
    walk.prev = walk.walk;
    walk.walk = walk.next;
    Some(walk)
}

fn erase_loop(maze: &mut maze::Maze, mut walk: Loop) {
    while walk.walk != walk.root {
        *maze.get_mut(walk.walk.row, walk.walk.col) &= !WALK_BIT;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        let next = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
        walk.walk = next;
    }
}

fn erase_loop_animated(maze: &mut maze::Maze, mut walk: Loop, speed: build::SpeedUnit) {
    while walk.walk != walk.root {
        *maze.get_mut(walk.walk.row, walk.walk.col) &= !WALK_BIT;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        let half: &'static maze::Point = backtrack_half_step(maze, &walk.walk);
        let half_step = maze::Point {
            row: walk.walk.row + half.row,
            col: walk.walk.col + half.col,
        };
        let next = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        *maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
        build::flush_cursor_maze_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(speed));
        build::flush_cursor_maze_coordinate(maze, walk.walk);
        thread::sleep(time::Duration::from_micros(speed));
        walk.walk = next;
    }
}

fn erase_mini_loop_animated(maze: &mut maze::Maze, mut walk: Loop, speed: build::SpeedUnit) {
    while walk.walk != walk.root {
        *maze.get_mut(walk.walk.row, walk.walk.col) &= !WALK_BIT;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        let half: &'static maze::Point = backtrack_half_step(maze, &walk.walk);
        let half_step = maze::Point {
            row: walk.walk.row + half.row,
            col: walk.walk.col + half.col,
        };
        let next = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        *maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
        build::flush_mini_backtracker_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(speed));
        build::flush_mini_backtracker_coordinate(maze, walk.walk);
        thread::sleep(time::Duration::from_micros(speed));
        walk.walk = next;
    }
}

fn connect_walk(maze: &mut maze::Maze, mut walk: maze::Point) {
    while (maze.get(walk.row, walk.col) & build::MARKERS_MASK) != 0 {
        let dir: &'static maze::Point = backtrack_point(maze, &walk);
        let next = maze::Point {
            row: walk.row + dir.row,
            col: walk.col + dir.col,
        };
        build_with_marks(maze, walk, next);
        *maze.get_mut(walk.row, walk.col) &= !build::MARKERS_MASK;
        walk = next;
    }
    *maze.get_mut(walk.row, walk.col) &= !build::MARKERS_MASK;
    *maze.get_mut(walk.row, walk.col) &= !WALK_BIT;
    build::build_wall_line(maze, walk);
}

fn connect_walk_animated(maze: &mut maze::Maze, mut walk: maze::Point, speed: build::SpeedUnit) {
    while (maze.get(walk.row, walk.col) & build::MARKERS_MASK) != 0 {
        let dir: &'static maze::Point = backtrack_point(maze, &walk);
        let half: &'static maze::Point = backtrack_half_step(maze, &walk);
        let half_step = maze::Point {
            row: walk.row + half.row,
            col: walk.col + half.col,
        };
        let next = maze::Point {
            row: walk.row + dir.row,
            col: walk.col + dir.col,
        };
        build_with_marks_animated(maze, walk, next, speed);
        *maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        *maze.get_mut(walk.row, walk.col) &= !build::MARKERS_MASK;
        build::flush_cursor_maze_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(speed));
        build::flush_cursor_maze_coordinate(maze, walk);
        thread::sleep(time::Duration::from_micros(speed));
        walk = next;
    }
    *maze.get_mut(walk.row, walk.col) &= !build::MARKERS_MASK;
    *maze.get_mut(walk.row, walk.col) &= !WALK_BIT;
    build::flush_cursor_maze_coordinate(maze, walk);
    thread::sleep(time::Duration::from_micros(speed));
    build::build_wall_line_animated(maze, walk, speed);
}

fn connect_mini_walk_animated(
    maze: &mut maze::Maze,
    mut walk: maze::Point,
    speed: build::SpeedUnit,
) {
    while (maze.get(walk.row, walk.col) & build::MARKERS_MASK) != 0 {
        let dir: &'static maze::Point = backtrack_point(maze, &walk);
        let half: &'static maze::Point = backtrack_half_step(maze, &walk);
        let half_step = maze::Point {
            row: walk.row + half.row,
            col: walk.col + half.col,
        };
        let next = maze::Point {
            row: walk.row + dir.row,
            col: walk.col + dir.col,
        };
        build_with_mini_marks_animated(maze, walk, next, speed);
        *maze.get_mut(half_step.row, half_step.col) &= !build::MARKERS_MASK;
        *maze.get_mut(walk.row, walk.col) &= !build::MARKERS_MASK;
        build::flush_mini_backtracker_coordinate(maze, half_step);
        thread::sleep(time::Duration::from_micros(speed));
        build::flush_mini_backtracker_coordinate(maze, walk);
        thread::sleep(time::Duration::from_micros(speed));
        walk = next;
    }
    *maze.get_mut(walk.row, walk.col) &= !build::MARKERS_MASK;
    *maze.get_mut(walk.row, walk.col) &= !WALK_BIT;
    build::flush_mini_backtracker_coordinate(maze, walk);
    thread::sleep(time::Duration::from_micros(speed));
    build::build_mini_wall_line_animated(maze, walk, speed);
}

fn build_with_marks(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
    let mut wall = cur;
    if next.row < cur.row {
        wall.row -= 1;
    } else if next.row > cur.row {
        wall.row += 1;
    } else if next.col < cur.col {
        wall.col -= 1;
    } else if next.col > cur.col {
        wall.col += 1;
    } else {
        print::maze_panic!("Wall break error. Step through wall didn't work");
    }
    *maze.get_mut(cur.row, cur.col) &= !WALK_BIT;
    *maze.get_mut(next.row, next.col) &= !WALK_BIT;
    build::build_wall_line(maze, cur);
    build::build_wall_line(maze, wall);
    build::build_wall_line(maze, next);
}

fn build_with_marks_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: build::SpeedUnit,
) {
    let mut wall = cur;
    if next.row < cur.row {
        wall.row -= 1;
    } else if next.row > cur.row {
        wall.row += 1;
    } else if next.col < cur.col {
        wall.col -= 1;
    } else if next.col > cur.col {
        wall.col += 1;
    } else {
        print::maze_panic!(
            "Wall break error. Step through wall didn't work: cur {:?}, next {:?}",
            cur,
            next
        );
    }
    *maze.get_mut(cur.row, cur.col) &= !WALK_BIT;
    *maze.get_mut(next.row, next.col) &= !WALK_BIT;
    build::build_wall_line_animated(maze, cur, speed);
    build::build_wall_line_animated(maze, wall, speed);
    build::build_wall_line_animated(maze, next, speed);
}

fn build_with_mini_marks_animated(
    maze: &mut maze::Maze,
    cur: maze::Point,
    next: maze::Point,
    speed: build::SpeedUnit,
) {
    let mut wall = cur;
    if next.row < cur.row {
        wall.row -= 1;
    } else if next.row > cur.row {
        wall.row += 1;
    } else if next.col < cur.col {
        wall.col -= 1;
    } else if next.col > cur.col {
        wall.col += 1;
    } else {
        print::maze_panic!(
            "Wall break error. Step through wall didn't work: cur {:?}, next {:?}",
            cur,
            next
        );
    }
    *maze.get_mut(cur.row, cur.col) &= !WALK_BIT;
    *maze.get_mut(next.row, next.col) &= !WALK_BIT;
    build::build_mini_wall_line_animated(maze, cur, speed);
    build::build_mini_wall_line_animated(maze, wall, speed);
    build::build_mini_wall_line_animated(maze, next, speed);
}

fn is_valid_step(maze: &maze::Maze, next: maze::Point, prev: maze::Point) -> bool {
    next.row >= 0
        && next.row < maze.row_size()
        && next.col >= 0
        && next.col < maze.col_size()
        && next != prev
}

fn backtrack_point(maze: &maze::Maze, walk: &maze::Point) -> &'static maze::Point {
    &build::BACKTRACKING_POINTS
        [((maze.get(walk.row, walk.col) & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize]
}

fn backtrack_half_step(maze: &maze::Maze, walk: &maze::Point) -> &'static maze::Point {
    &build::BACKTRACKING_HALF_POINTS
        [((maze.get(walk.row, walk.col) & build::MARKERS_MASK) >> build::MARKER_SHIFT) as usize]
}

fn found_loop(maze: &maze::Maze, p: maze::Point) -> bool {
    (maze.get(p.row, p.col) & WALK_BIT) != 0
}
