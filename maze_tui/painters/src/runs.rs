use crate::rgb;
use maze;

use std::collections::VecDeque;
use std::thread;

use rand::{thread_rng, Rng};

struct RunPoint {
    len: u64,
    prev: maze::Point,
    cur: maze::Point,
}

///
/// Data only measurements.
///

pub fn paint_run_lengths(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("Lock panic."),
    };
    let row_mid = lk.maze.rows() / 2;
    let col_mid = lk.maze.cols() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = monitor::MaxMap::new(start, 0);
    let mut bfs = VecDeque::from([RunPoint {
        len: 0,
        prev: start,
        cur: start,
    }]);
    *lk.maze.get_mut(start.row, start.col) |= rgb::MEASURED;
    while let Some(cur) = bfs.pop_front() {
        if cur.len > map.max {
            map.max = cur.len;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.cur.row + p.row,
                col: cur.cur.col + p.col,
            };
            if lk.maze.wall_at(next.row, next.col)
                || (lk.maze.get(next.row, next.col) & rgb::MEASURED) != 0
            {
                continue;
            }
            let next_run_len =
                if (next.row).abs_diff(cur.prev.row) == (next.col).abs_diff(cur.prev.col) {
                    1
                } else {
                    cur.len + 1
                };
            *lk.maze.get_mut(next.row, next.col) |= rgb::MEASURED;
            map.distances.insert(next, next_run_len);
            bfs.push_back(RunPoint {
                len: next_run_len,
                prev: cur.cur,
                cur: next,
            });
        }
    }
    painter(&mut lk.maze, &map);
}

fn painter(maze: &mut maze::Maze, map: &monitor::MaxMap) {
    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    for r in 0..maze.rows() {
        for c in 0..maze.cols() {
            let cur = maze::Point { row: r, col: c };
            if let Some(dist) = map.distances.get(&cur) {
                let intensity = (map.max - dist) as f64 / map.max as f64;
                let dark = (255f64 * intensity) as u8;
                let bright = 128 + (127f64 * intensity) as u8;
                let mut c: rgb::Rgb = [dark, dark, dark];
                c[rand_color_choice] = bright;
                *maze.get_mut(cur.row, cur.col) |= ((c[0] as u32) << rgb::RED_SHIFT)
                    | ((c[1] as u32) << rgb::GREEN_SHIFT)
                    | (c[2] as u32);
            }
        }
    }
}

///
/// History based solver.
///

pub fn paint_run_lengths_history(monitor: monitor::MazeMonitor) {
    let start: maze::Point = if let Ok(mut lk) = monitor.lock() {
        let row_mid = lk.maze.rows() / 2;
        let col_mid = lk.maze.cols() / 2;
        let start = maze::Point {
            row: row_mid + 1 - (row_mid % 2),
            col: col_mid + 1 - (col_mid % 2),
        };
        lk.map.distances.insert(start, 0);
        let mut bfs = VecDeque::from([RunPoint {
            len: 0,
            prev: start,
            cur: start,
        }]);
        *lk.maze.get_mut(start.row, start.col) |= rgb::MEASURED;
        while let Some(cur) = bfs.pop_front() {
            if cur.len > lk.map.max {
                lk.map.max = cur.len;
            }
            for &p in maze::CARDINAL_DIRECTIONS.iter() {
                let next = maze::Point {
                    row: cur.cur.row + p.row,
                    col: cur.cur.col + p.col,
                };
                if (lk.maze.get(next.row, next.col) & maze::PATH_BIT) == 0
                    || (lk.maze.get(next.row, next.col) & rgb::MEASURED) != 0
                {
                    continue;
                }
                let next_run_len =
                    if (next.row).abs_diff(cur.prev.row) == (next.col).abs_diff(cur.prev.col) {
                        1
                    } else {
                        cur.len + 1
                    };
                *lk.maze.get_mut(next.row, next.col) |= rgb::MEASURED;
                lk.map.distances.insert(next, next_run_len);
                bfs.push_back(RunPoint {
                    len: next_run_len,
                    prev: cur.cur,
                    cur: next,
                });
            }
        }
        start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    let mut handles = Vec::with_capacity(rgb::NUM_PAINTERS);
    for painter in 0..rgb::NUM_PAINTERS - 1 {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            painter_history(
                monitor_clone,
                rgb::ThreadGuide {
                    bias: painter,
                    color_i: rand_color_choice,
                    cache: rgb::MEASURED_MASKS[painter],
                    p: start,
                },
            );
        }));
    }
    painter_history(
        monitor.clone(),
        rgb::ThreadGuide {
            bias: rgb::NUM_PAINTERS - 1,
            color_i: rand_color_choice,
            cache: rgb::MEASURED_MASKS[rgb::NUM_PAINTERS - 1],
            p: start,
        },
    );
    for h in handles {
        h.join().expect("Error joining a thread.");
    }
}

fn painter_history(monitor: monitor::MazeMonitor, guide: rgb::ThreadGuide) {
    let mut bfs = VecDeque::from([guide.p]);
    while let Some(cur) = bfs.pop_front() {
        match monitor.lock() {
            Ok(mut lk) => {
                if lk.count == lk.map.distances.len() {
                    return;
                }
                let run = lk
                    .map
                    .distances
                    .get(&cur)
                    .expect("Could not find map entry?");
                let before = lk.maze.get(cur.row, cur.col);
                if !rgb::has_paint_vals(before) {
                    let intensity = (lk.map.max - run) as f64 / lk.map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut c: rgb::Rgb = [dark, dark, dark];
                    c[guide.color_i] = bright;
                    lk.maze.solve_history.push(maze::Delta {
                        id: cur,
                        before,
                        after: before
                            | ((c[0] as u32) << rgb::RED_SHIFT)
                            | ((c[1] as u32) << rgb::GREEN_SHIFT)
                            | (c[2] as u32),
                        burst: 1,
                    });
                    *lk.maze.get_mut(cur.row, cur.col) |= ((c[0] as u32) << rgb::RED_SHIFT)
                        | ((c[1] as u32) << rgb::GREEN_SHIFT)
                        | (c[2] as u32);
                    lk.count += 1;
                }
            }
            Err(p) => print::maze_panic!("Thread panicked with lock: {}", p),
        };
        let mut i = guide.bias;
        for _ in 0..rgb::NUM_DIRECTIONS {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Panic with lock: {}", p),
                Ok(mut lk) => {
                    let nxt = lk.maze.get(next.row, next.col);
                    let seen = (nxt & guide.cache) == 0;
                    let is_path = maze::is_path(nxt);
                    if seen && is_path {
                        *lk.maze.get_mut(next.row, next.col) |= guide.cache;
                    }
                    seen && is_path
                }
            } {
                bfs.push_back(next);
            }
            i = (i + 1) % rgb::NUM_PAINTERS;
        }
    }
}
