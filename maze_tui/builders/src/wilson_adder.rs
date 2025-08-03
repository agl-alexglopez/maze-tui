use crate::build;
use maze;
use print;

use rand::{seq::SliceRandom, thread_rng, Rng};

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

///
/// Data only maze generator
///
pub fn generate_maze(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::build_wall_outline(&mut lk.maze);
    let mut rng = thread_rng();
    let mut cur = RandomWalk {
        prev_row_start: 2,
        prev: maze::Point { row: 0, col: 0 },
        walk: maze::Point {
            row: 2 * (rng.gen_range(2..lk.maze.rows() - 1) / 2),
            col: 2 * (rng.gen_range(2..lk.maze.cols() - 1) / 2),
        },
        next: maze::Point { row: 0, col: 0 },
    };
    let mut indices: [usize; 4] = [0, 1, 2, 3];
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
            cur = match complete_walk(&mut lk.maze, cur) {
                None => return,
                Some(new_walk) => new_walk,
            };
            continue 'walking;
        }
    }
}

fn complete_walk(maze: &mut maze::Maze, mut walk: RandomWalk) -> Option<RandomWalk> {
    if build::has_builder_bit(maze, walk.next) {
        build_with_marks(maze, walk.walk, walk.next);
        connect_walk(maze, walk.walk);
        if let Some(point) =
            build::choose_point_from_row_start(maze, walk.prev_row_start, build::ParityPoint::Even)
        {
            walk.prev_row_start = point.row;
            walk.walk = point;
            *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
            walk.prev = maze::Point { row: 0, col: 0 };
            return Some(walk);
        }
        return None;
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

///
/// History based generator for animation and playback.
///
pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::build_wall_outline_history(&mut lk.maze);
    let mut rng = thread_rng();
    let mut cur = RandomWalk {
        prev_row_start: 2,
        prev: maze::Point { row: 0, col: 0 },
        walk: maze::Point {
            row: 2 * (rng.gen_range(2..lk.maze.rows() - 1) / 2),
            col: 2 * (rng.gen_range(2..lk.maze.cols() - 1) / 2),
        },
        next: maze::Point { row: 0, col: 0 },
    };
    let mut indices: [usize; 4] = [0, 1, 2, 3];
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
            cur = match complete_walk_history(&mut lk.maze, cur) {
                None => return,
                Some(new_walk) => new_walk,
            };
            continue 'walking;
        }
    }
}

fn complete_walk_history(maze: &mut maze::Maze, mut walk: RandomWalk) -> Option<RandomWalk> {
    if build::has_builder_bit(maze, walk.next) {
        bridge_gap_history(maze, walk.walk, walk.next);
        connect_walk_history(maze, walk.walk);
        if let Some(point) =
            build::choose_point_from_row_start(maze, walk.prev_row_start, build::ParityPoint::Even)
        {
            walk.prev_row_start = point.row;
            walk.walk = point;
            *maze.get_mut(walk.walk.row, walk.walk.col) &= !build::MARKERS_MASK;
            walk.prev = maze::Point { row: 0, col: 0 };
            return Some(walk);
        }
        return None;
    }
    if found_loop(maze, walk.next) {
        erase_loop_history(
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
    mark_line_history(maze, walk.walk, walk.next);
    walk.prev = walk.walk;
    walk.walk = walk.next;
    Some(walk)
}

fn erase_loop_history(maze: &mut maze::Maze, mut walk: Loop) {
    while walk.walk != walk.root {
        let walk_square = maze.get(walk.walk.row, walk.walk.col);
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        let half: &'static maze::Point = backtrack_half_step(maze, &walk.walk);
        let half_step = maze::Point {
            row: walk.walk.row + half.row,
            col: walk.walk.col + half.col,
        };
        let half_square = maze.get(half_step.row, half_step.col);
        let next = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        maze.build_history.push(maze::Delta {
            id: walk.walk,
            before: walk_square,
            after: (((walk_square & !WALK_BIT) & !maze::WALL_MASK) & !build::MARKERS_MASK)
                | maze::PATH_BIT,
            burst: 1,
        });
        maze.build_history.push(maze::Delta {
            id: half_step,
            before: half_square,
            after: ((half_square & !build::MARKERS_MASK) & !maze::WALL_MASK) | maze::PATH_BIT,
            burst: 1,
        });
        *maze.get_mut(half_step.row, half_step.col) =
            ((half_square & !build::MARKERS_MASK) & !maze::WALL_MASK) | maze::PATH_BIT;
        *maze.get_mut(walk.walk.row, walk.walk.col) =
            (((walk_square & !WALK_BIT) & !maze::WALL_MASK) & !build::MARKERS_MASK)
                | maze::PATH_BIT;
        walk.walk = next;
    }
}

fn connect_walk_history(maze: &mut maze::Maze, mut walk: maze::Point) {
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
        build_walk_line_history(maze, walk);
        build_walk_line_history(maze, half_step);
        walk = next;
    }
    build_walk_line_history(maze, walk);
}

pub fn mark_line_history(maze: &mut maze::Maze, walk: maze::Point, next: maze::Point) {
    let mut wall = walk;
    let next_before = maze.get(next.row, next.col);
    let wall_before = if next.row > walk.row {
        wall.row += 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) = (before | build::FROM_NORTH) & !maze::PATH_BIT;
        *maze.get_mut(next.row, next.col) = (next_before | build::FROM_NORTH) & !maze::PATH_BIT;
        before
    } else if next.row < walk.row {
        wall.row -= 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) = (before | build::FROM_SOUTH) & !maze::PATH_BIT;
        *maze.get_mut(next.row, next.col) = (next_before | build::FROM_SOUTH) & !maze::PATH_BIT;
        before
    } else if next.col < walk.col {
        wall.col -= 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) = (before | build::FROM_EAST) & !maze::PATH_BIT;
        *maze.get_mut(next.row, next.col) = (next_before | build::FROM_EAST) & !maze::PATH_BIT;
        before
    } else if next.col > walk.col {
        wall.col += 1;
        let before = maze.get(wall.row, wall.col);
        *maze.get_mut(wall.row, wall.col) = (before | build::FROM_WEST) & !maze::PATH_BIT;
        *maze.get_mut(next.row, next.col) = (next_before | build::FROM_WEST) & !maze::PATH_BIT;
        before
    } else {
        print::maze_panic!("next cannot be equal to walk");
    };
    maze.build_history.push(maze::Delta {
        id: wall,
        before: wall_before,
        after: maze.get(wall.row, wall.col),
        burst: 1,
    });
    maze.build_history.push(maze::Delta {
        id: next,
        before: next_before,
        after: maze.get(next.row, next.col),
        burst: 1,
    });
}

fn build_walk_line_history(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall_changes = [maze::Delta::default(); 5];
    let mut burst = 1;
    let mut wall: maze::WallLine = 0b0;
    let square = maze.get(p.row, p.col);
    if p.row > 0 && maze.wall_at(p.row - 1, p.col) {
        let neighbor = maze.get(p.row - 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: neighbor,
            after: (neighbor | maze::SOUTH_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) |= maze::SOUTH_WALL;
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.rows() && maze.wall_at(p.row + 1, p.col) {
        let neighbor = maze.get(p.row + 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: neighbor,
            after: (neighbor | maze::NORTH_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) |= maze::NORTH_WALL;
        wall |= maze::SOUTH_WALL;
    }
    if p.col > 0 && maze.wall_at(p.row, p.col - 1) {
        let neighbor = maze.get(p.row, p.col - 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: neighbor,
            after: (neighbor | maze::EAST_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) |= maze::EAST_WALL;
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.cols() && maze.wall_at(p.row, p.col + 1) {
        let neighbor = maze.get(p.row, p.col + 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: neighbor,
            after: (neighbor | maze::WEST_WALL),
            burst: burst + 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) |= maze::WEST_WALL;
        wall |= maze::EAST_WALL;
    }
    wall_changes[0] = maze::Delta {
        id: p,
        before: square,
        after: ((square | wall | build::BUILDER_BIT) & !maze::PATH_BIT)
            & !WALK_BIT
            & !build::MARKERS_MASK,
        burst,
    };
    *maze.get_mut(p.row, p.col) =
        ((square | wall | build::BUILDER_BIT) & !maze::PATH_BIT) & !WALK_BIT & !build::MARKERS_MASK;
    maze.build_history.push_burst(&wall_changes[0..burst]);
}

fn bridge_gap_history(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point) {
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
    build_walk_line_history(maze, wall);
}

///
/// Logic utilities available to all.
///
fn is_valid_step(maze: &maze::Maze, next: maze::Point, prev: maze::Point) -> bool {
    next.row >= 0
        && next.row < maze.rows()
        && next.col >= 0
        && next.col < maze.cols()
        && next != prev
}

fn backtrack_point(maze: &maze::Maze, walk: &maze::Point) -> &'static maze::Point {
    &build::BACKTRACKING_POINTS[(maze.get(walk.row, walk.col) & build::MARKERS_MASK) as usize]
}

fn backtrack_half_step(maze: &maze::Maze, walk: &maze::Point) -> &'static maze::Point {
    &build::BACKTRACKING_HALF_POINTS[(maze.get(walk.row, walk.col) & build::MARKERS_MASK) as usize]
}

fn found_loop(maze: &maze::Maze, p: maze::Point) -> bool {
    (maze.get(p.row, p.col) & WALK_BIT) != 0
}
