use builders;
use maze;
use print;

use std::collections::{HashMap, VecDeque};
use std::io::{stdout, Write};
use std::{thread, time};

use rand::{thread_rng, Rng};

type SpeedUnit = u64;

struct Rgb {
    ch: [u8;3],
}

struct DistanceMap {
    max: u64,
    distances: HashMap<maze::Point, u64>,
}

impl DistanceMap {
    fn new(p: maze::Point, dist: u64) -> Self {
        Self { max: dist, distances: HashMap::from([(p, dist)]) }
    }
}

const RGB_ESCAPE: &str = "\x1b[38;2;";
const R: usize = 0;
const G: usize = 1;
const B: usize = 2;
const PAINTED_BIT: maze::Square = 0b1_0000;
const ANIMATION_SPEEDS: [SpeedUnit;8] = [0, 10000, 5000, 2000, 1000, 500, 250, 50];

pub fn paint_distance_from_center(maze: maze::BoxMaze) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = DistanceMap::new(start, 0);
    let mut bfs = VecDeque::from([(start, 0u64)]);
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > map.max {
            map.max = cur.1;
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
            match map.distances.get_mut(&next) {
                Some(dist) => {
                    if cur_dist < *dist {
                        *dist = cur_dist;
                    }
                }
                None => {
                    map.distances.insert(next, cur_dist + 1);
                    bfs.push_back((next, cur_dist + 1));
                }
            };
        }
    }
    print_distances(maze, &map);
}

pub fn animate_distance_from_center(maze: maze::BoxMaze, speed: speed::Speed) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = DistanceMap::new(start, 0);
    let mut bfs = VecDeque::from([(start, 0u64)]);
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > map.max {
            map.max = cur.1;
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
            match map.distances.get_mut(&next) {
                Some(dist) => {
                    if cur_dist < *dist {
                        *dist = cur_dist;
                    }
                }
                None => {
                    map.distances.insert(next, cur_dist + 1);
                    bfs.push_back((next, cur_dist + 1));
                }
            };
        }
    }
    animate_distances(maze, &map, ANIMATION_SPEEDS[speed as usize]);
}

fn animate_distances(mut maze: maze::BoxMaze, map: &DistanceMap, animation: SpeedUnit) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut bfs = VecDeque::from([start]);
    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    while let Some(cur) = bfs.pop_front() {
        maze[cur.row as usize][cur.col as usize] |= PAINTED_BIT;
        match map.distances.get(&cur) {
            Some(dist) => {
                let intensity = (map.max - dist) as f64 / map.max as f64;
                let dark = (255f64 * intensity) as u8;
                let bright = 128 + (127f64 * intensity) as u8;
                let mut color = Rgb {ch: [dark, dark, dark] };
                color.ch[rand_color_choice] = bright;
                print_rgb(
                    color,
                    cur,
                );
                stdout().flush().expect("Couldn't flush cursor");
                thread::sleep(time::Duration::from_micros(animation));
            }
            None => print::maze_panic!("Error finding current square's distance"),
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if (maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                || (maze[next.row as usize][next.col as usize] & PAINTED_BIT) != 0 {
                continue;
            }
            bfs.push_back(next);
        }
    }
    print::set_cursor_position(maze::Point{row: maze.row_size(), col: maze.col_size()});
    println!();
}
fn print_distances(maze: maze::BoxMaze, map: &DistanceMap) {
    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            let cur = maze::Point { row: r, col: c };
            match map.distances.get(&cur) {
                Some(dist) => {
                    let intensity = (map.max - dist) as f64 / map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut color = Rgb {ch: [dark, dark, dark]};
                    color.ch[rand_color_choice] = bright;
                    print_rgb(
                        color,
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
                + &rgb.ch[R].to_string()
                + ";"
                + &rgb.ch[G].to_string()
                + ";"
                + &rgb.ch[B].to_string()
                + "m"
                + "█"
                + "\x1b[0m"
        )
    );
}
