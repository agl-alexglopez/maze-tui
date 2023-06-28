use builders;
use maze;
use print;

use std::collections::{HashMap, VecDeque};

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

const RGB_ESCAPE: &str = "\x1b[38;2;";

fn main() {
    let args = maze::MazeArgs::default();
    let mut maze = maze::Maze::new(args);
    builders::eller::generate_maze(&mut maze);
    builders::build::clear_and_flush_grid(&maze);
    paint_distance_from_center(&maze);
}

fn paint_distance_from_center(maze: &maze::Maze) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut distances: HashMap<maze::Point, u64> = HashMap::from([(start, 0)]);
    let mut bfs = VecDeque::from([(start, 0u64)]);
    let mut max_dist = 0;
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > max_dist {
            max_dist = cur.1;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.0.row + p.row,
                col: cur.0.col + p.col,
            };
            if (maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0 {
                continue;
            }
            let cur_dist = cur.1;
            match distances.get_mut(&next) {
                Some(dist) => {
                    if cur_dist < *dist {
                        *dist = cur_dist;
                    }
                }
                None => {
                    distances.insert(next, cur_dist + 1);
                    bfs.push_back((next, cur_dist + 1));
                }
            };
        }
    }
    print_distances(maze, &distances, max_dist);
}

fn print_distances(maze: &maze::Maze, distances: &HashMap<maze::Point, u64>, max_dist: u64) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            let cur = maze::Point { row: r, col: c };
            match distances.get(&cur) {
                Some(dist) => {
                    let intensity = (max_dist - dist) as f64 / max_dist as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    print_rgb(
                        Rgb {
                            r: dark,
                            g: dark,
                            b: bright,
                        },
                        cur,
                    );
                }
                None => builders::build::print_square(&maze, cur),
            }
        }
    }
    println!();
}

fn print_rgb(rgb: Rgb, p: maze::Point) {
    print::set_cursor_position(p);
    print!(
        "{}",
        String::from(
            RGB_ESCAPE.to_owned()
                + &rgb.r.to_string()
                + ";"
                + &rgb.g.to_string()
                + ";"
                + &rgb.b.to_string()
                + "m"
                + "█"
                + "\x1b[0m"
        )
    );
}
