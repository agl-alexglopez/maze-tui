use crate::rgb;
use builders::build::print_square;
use maze;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::{thread, time};

use rand::{thread_rng, Rng};

struct DistanceMap {
    max: u64,
    distances: HashMap<maze::Point, u64>,
}

type BoxMap = Box<DistanceMap>;

impl DistanceMap {
    fn new(p: maze::Point, dist: u64) -> Self {
        Self {
            max: dist,
            distances: HashMap::from([(p, dist)]),
        }
    }
}

struct ThreadGuide {
    bias: usize,
    color_i: usize,
    p: maze::Point,
}

struct BfsPainter {
    maze: maze::BoxMaze,
    map: BoxMap,
    count: usize,
}

impl BfsPainter {
    fn new(box_maze: maze::BoxMaze, dist_map: BoxMap) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            maze: box_maze,
            map: dist_map,
            count: 0,
        }))
    }
}

type BfsMonitor = Arc<Mutex<BfsPainter>>;

pub fn paint_distance_from_center(mut maze: maze::BoxMaze) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = DistanceMap::new(start, 0);
    let mut bfs = VecDeque::from([(start, 0u64)]);
    maze[start.row as usize][start.col as usize] |= rgb::MEASURE;
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > map.max {
            map.max = cur.1;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.0.row + p.row,
                col: cur.0.col + p.col,
            };
            if (maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                || (maze[next.row as usize][next.col as usize] & rgb::MEASURE) != 0
            {
                continue;
            }
            maze[next.row as usize][next.col as usize] |= rgb::MEASURE;
            map.distances.insert(next, cur.1 + 1);
            bfs.push_back((next, cur.1 + 1));
        }
    }
    painter(maze, &map);
    println!();
}

pub fn animate_distance_from_center(mut maze: maze::BoxMaze, speed: speed::Speed) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = DistanceMap::new(start, 0);
    let mut bfs = VecDeque::from([(start, 0u64)]);
    maze[start.row as usize][start.col as usize] |= rgb::MEASURE;
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > map.max {
            map.max = cur.1;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.0.row + p.row,
                col: cur.0.col + p.col,
            };
            if (maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                || (maze[next.row as usize][next.col as usize] & rgb::MEASURE) != 0
            {
                continue;
            }
            maze[next.row as usize][next.col as usize] |= rgb::MEASURE;
            map.distances.insert(next, cur.1 + 1);
            bfs.push_back((next, cur.1 + 1));
        }
    }

    let box_map = Box::new(map);
    let monitor = BfsPainter::new(maze, box_map);
    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    let mut handles = Vec::with_capacity(rgb::NUM_PAINTERS);
    let animation = rgb::ANIMATION_SPEEDS[speed as usize];
    for painter in 0..rgb::NUM_PAINTERS {
        let mut monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            painter_animated(
                &mut monitor_clone,
                ThreadGuide {
                    bias: painter,
                    color_i: rand_color_choice,
                    p: start,
                },
                animation,
            );
        }));
    }
    for h in handles {
        h.join().expect("Error joining a thread.");
    }
    match monitor.lock() {
        Ok(lk) => {
            print::set_cursor_position(maze::Point {
                row: lk.maze.row_size(),
                col: lk.maze.col_size(),
            });
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked: {}", p),
    };
}

// Private Helper Functions-----------------------------------------------------------------------

fn painter(maze: maze::BoxMaze, map: &DistanceMap) {
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
                    let mut color = rgb::Rgb {
                        ch: [dark, dark, dark],
                    };
                    color.ch[rand_color_choice] = bright;
                    rgb::print_rgb(color, cur);
                }
                None => print_square(&maze, cur),
            }
        }
    }
    println!();
}

fn painter_animated(monitor: &mut BfsMonitor, guide: ThreadGuide, animation: rgb::SpeedUnit) {
    let mut seen = HashSet::from([guide.p]);
    let mut bfs = VecDeque::from([guide.p]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => {
                if lk.count == lk.map.distances.len() {
                    return;
                }
                let dist = lk
                    .map
                    .distances
                    .get(&cur)
                    .expect("Could not find map entry?");
                if (lk.maze[cur.row as usize][cur.col as usize] & rgb::PAINT) == 0 {
                    let intensity = (lk.map.max - dist) as f64 / lk.map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut color = rgb::Rgb {
                        ch: [dark, dark, dark],
                    };
                    color.ch[guide.color_i] = bright;
                    rgb::animate_rgb(color, cur);
                    lk.maze[cur.row as usize][cur.col as usize] |= rgb::PAINT;
                    lk.count += 1;
                }
            }
            Err(p) => print::maze_panic!("Thread panicked with lock: {}", p),
        };
        thread::sleep(time::Duration::from_micros(animation));
        let mut i = guide.bias;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Panic with lock: {}", p),
                Ok(lk) => {
                    (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                }
            } && !seen.contains(&next) {
                seen.insert(next);
                bfs.push_back(next);
            }
            i = (i + 1) % rgb::NUM_PAINTERS;
            i != guide.bias
        } {}
    }
}
