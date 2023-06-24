use crate::build;
use crate::maze;

use rand::{seq::SliceRandom, thread_rng, Rng};

const WALK_BIT: maze::Square = 0b0100_0000_0000_0000;

struct Loop {
    walk: maze::Point,
    root: maze::Point,
}

#[derive(Clone, Copy)]
struct RandomWalk {
    prev: maze::Point,
    walk: maze::Point,
    next: maze::Point,
}

// Public Functions-------------------------------------------------------------------------------

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    let mut rng = thread_rng();
    let start = maze::Point {
        row: 2 * (rng.gen_range(2..maze.row_size() - 1) / 2) + 1,
        col: 2 * (rng.gen_range(2..maze.col_size() - 1) / 2) + 1,
    };
    build::build_path(maze, start);
    maze[start.row as usize][start.col as usize] |= maze::BUILDER_BIT;
    let cur = RandomWalk {
        prev: maze::Point { row: 0, col: 0 },
        walk: maze::Point { row: 1, col: 1 },
        next: maze::Point { row: 0, col: 0 },
    };
}

pub fn animate_maze(maze: &mut maze::Maze, speed: build::BuilderSpeed) {}

// Private Functions------------------------------------------------------------------------------

fn complete_walk(maze: &mut maze::Maze, mut walk: RandomWalk) -> Option<RandomWalk> {
    if build::has_builder_bit(maze, walk.next) {
        build_with_marks(maze, walk.walk, walk.next);
        connect_walk(maze, walk.walk);
        match build::choose_arbitrary_point(maze, build::ParityPoint::Odd) {
            Some(point) => {
                walk.walk = point;
                maze[walk.walk.row as usize][walk.walk.col as usize] &= !build::MARKERS_MASK;
                walk.prev = maze::Point { row: 0, col: 0 };
                Some(walk)
            }
            None => None,
        };
    }
    if (maze[walk.next.row as usize][walk.next.col as usize] & WALK_BIT) != 0 {
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
        maze[walk.walk.row as usize][walk.walk.col as usize] &= !WALK_BIT;
        let dir: &'static maze::Point = backtrack_point(maze, &walk.walk);
        let next = maze::Point {
            row: walk.walk.row + dir.row,
            col: walk.walk.col + dir.col,
        };
        maze[next.row as usize][next.col as usize] &= !build::MARKERS_MASK;
        walk.walk = next;
    }
}

fn connect_walk(maze: &mut maze::Maze, mut walk: maze::Point) {
    while (maze[walk.row as usize][walk.col as usize]) != 0 {
        let dir: &'static maze::Point = backtrack_point(maze, &walk);
        let next = maze::Point {
            row: walk.row + dir.row,
            col: walk.col + dir.col,
        };
        build_with_marks(maze, walk, next);
        maze[walk.row as usize][walk.col as usize] &= !build::MARKERS_MASK;
        walk = next;
    }
    maze[walk.row as usize][walk.col as usize] &= !build::MARKERS_MASK;
    maze[walk.row as usize][walk.col as usize] &= !WALK_BIT;
    build::carve_path_walls(maze, walk);
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
        panic!("Wall break error. Step through wall didn't work");
    }
    maze[cur.row as usize][cur.col as usize] &= !WALK_BIT;
    maze[next.row as usize][next.col as usize] &= !WALK_BIT;
    build::carve_path_walls(maze, cur);
    build::carve_path_walls(maze, next);
    build::carve_path_walls(maze, wall);
}

fn is_valid_step(maze: &maze::Maze, next: maze::Point, prev: maze::Point) -> bool {
    next.row > 0
        && next.row < maze.row_size() - 1
        && next.col > 0
        && next.col < maze.col_size() - 1
        && next != prev
}

fn backtrack_point(maze: &maze::Maze, walk: &maze::Point) -> &'static maze::Point {
    &build::BACKTRACKING_POINTS[((maze[walk.row as usize][walk.col as usize] & build::MARKERS_MASK)
        >> build::MARKER_SHIFT) as usize]
}
