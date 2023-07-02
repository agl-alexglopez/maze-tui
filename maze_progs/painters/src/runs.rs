use maze;
use speed;
use builders::build::print_square;
use crate::rgb;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::{thread, time};

use rand::{thread_rng, Rng};

const IS_MEASURED_BIT: maze::Square = 0b1000_0000;

struct RunMap {
    max: u32,
    runs: HashMap<maze::Point, u32>,
}

struct RunPoint {
    len: u32,
    prev: maze::Point,
    cur: maze::Point,
}

type BoxMap = Box<RunMap>;

impl RunMap {
    fn new(p: maze::Point, run: u32) -> Self {
        Self { max: run, runs: HashMap::from([(p, run)]) }
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
    fn new(box_maze: maze::BoxMaze, run_map: BoxMap) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            maze: box_maze,
            map: run_map,
            count: 0,
        }))
    }
}

type BfsMonitor = Arc<Mutex<BfsPainter>>;

pub fn paint_run_lengths(mut maze: maze::BoxMaze) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = RunMap::new(start, 0);
    let mut bfs = VecDeque::from([RunPoint {len: 0, prev: start, cur: start}]);
    while let Some(cur) = bfs.pop_front() {
        maze[cur.cur.row as usize][cur.cur.col as usize] |= IS_MEASURED_BIT;
        if cur.len > map.max {
            map.max = cur.len;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.cur.row + p.row,
                col: cur.cur.col + p.col,
            };
            if (maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                || (maze[next.row as usize][next.col as usize] & IS_MEASURED_BIT) != 0 {
                continue;
            }
            let next_run_len = if (next.row).abs_diff(cur.prev.row) == (next.col).abs_diff(cur.prev.col) {
                1
            } else {
                cur.len + 1
            };
            map.runs.insert(next, next_run_len);
            bfs.push_back(RunPoint {len: next_run_len, prev: cur.cur, cur: next});
        }
    }
    painter(maze, &map);
    println!();
}

pub fn animate_run_lengths(mut maze: maze::BoxMaze, speed: speed::Speed) {
    let row_mid = maze.row_size() / 2;
    let col_mid = maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = RunMap::new(start, 0);
    let mut bfs = VecDeque::from([RunPoint {len: 0, prev: start, cur: start}]);
    while let Some(cur) = bfs.pop_front() {
        maze[cur.cur.row as usize][cur.cur.col as usize] |= IS_MEASURED_BIT;
        if cur.len > map.max {
            map.max = cur.len;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.cur.row + p.row,
                col: cur.cur.col + p.col,
            };
            if (maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                || (maze[next.row as usize][next.col as usize] & IS_MEASURED_BIT) != 0 {
                continue;
            }
            let next_run_len = if (next.row).abs_diff(cur.prev.row) == (next.col).abs_diff(cur.prev.col) {
                1
            } else {
                cur.len + 1
            };
            map.runs.insert(next, next_run_len);
            bfs.push_back(RunPoint {len: next_run_len, prev: cur.cur, cur: next});
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
            print::set_cursor_position(maze::Point{row: lk.maze.row_size(), col: lk.maze.col_size()});
            println!();
        }
        Err(p) => print::maze_panic!("Thread panicked: {}", p),
    };
}

// Private Helper Functions-----------------------------------------------------------------------

fn painter(maze: maze::BoxMaze, map: &RunMap) {
    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            let cur = maze::Point {row: r, col: c};
            match map.runs.get(&cur) {
                Some(run) => {
                    let intensity = (map.max - run) as f64 / map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut color = rgb::Rgb {ch: [dark, dark, dark]};
                    color.ch[rand_color_choice] = bright;
                    rgb::print_rgb(
                        color,
                        cur,
                    );
                }
                None => {
                    print_square(&maze, cur);
                }
            };
        }
    }
    println!();
}

fn painter_animated(monitor: &mut BfsMonitor, guide: ThreadGuide, animation: rgb::SpeedUnit) {
    let mut bfs = VecDeque::from([guide.p]);
    let mut seen: HashSet<maze::Point> = HashSet::from([guide.p]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => {
                if lk.count == lk.map.runs.len() {
                    return;
                }
                let run = lk.map.runs.get(&cur).expect("Could not find map entry?");
                if (lk.maze[cur.row as usize][cur.col as usize] & rgb::PAINTED_BIT) == 0 {
                    let intensity = (lk.map.max - run) as f64 / lk.map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut color = rgb::Rgb {ch: [dark, dark, dark]};
                    color.ch[guide.color_i] = bright;
                    rgb::animate_rgb(
                        color,
                        cur,
                    );
                    lk.maze[cur.row as usize][cur.col as usize] |= rgb::PAINTED_BIT;
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
            let cached = seen.contains(&next);
            let mut push_next = false;
            match monitor.lock() {
                Ok(lk) => {
                    push_next = (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0
                        && !cached;
                }
                Err(p) => print::maze_panic!("Panic with lock: {}, push next: {}", p, push_next),
            }
            if push_next {
                seen.insert(next);
                bfs.push_back(next);
            }
            i = (i + 1) % rgb::NUM_PAINTERS;
            i != guide.bias
        } {}
    }
}
