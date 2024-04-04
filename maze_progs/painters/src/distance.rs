use crate::rgb;
use maze;
use std::collections::VecDeque;

use std::thread;

use rand::{thread_rng, Rng};

///
/// Data only modifiers
///

pub fn paint_distance_from_center(monitor: monitor::MazeReceiver) {
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
    let mut bfs = VecDeque::from([(start, 0u64)]);
    *lk.maze.get_mut(start.row, start.col) |= rgb::MEASURED;
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > map.max {
            map.max = cur.1;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.0.row + p.row,
                col: cur.0.col + p.col,
            };
            if (lk.maze.get(next.row, next.col) & maze::PATH_BIT) == 0
                || (lk.maze.get(next.row, next.col) & rgb::MEASURED) != 0
            {
                continue;
            }
            *lk.maze.get_mut(next.row, next.col) |= rgb::MEASURED;
            map.distances.insert(next, cur.1 + 1);
            bfs.push_back((next, cur.1 + 1));
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
/// History based solvers.
///

pub fn paint_distance_from_center_history(monitor: monitor::MazeMonitor) {
    let start = if let Ok(mut lk) = monitor.lock() {
        let row_mid = lk.maze.rows() / 2;
        let col_mid = lk.maze.cols() / 2;
        let start = maze::Point {
            row: row_mid + 1 - (row_mid % 2),
            col: col_mid + 1 - (col_mid % 2),
        };
        lk.map.distances.insert(start, 0);
        let mut bfs = VecDeque::from([(start, 0u64)]);
        *lk.maze.get_mut(start.row, start.col) |= rgb::MEASURED;
        while let Some(cur) = bfs.pop_front() {
            if cur.1 > lk.map.max {
                lk.map.max = cur.1;
            }
            for &p in maze::CARDINAL_DIRECTIONS.iter() {
                let next = maze::Point {
                    row: cur.0.row + p.row,
                    col: cur.0.col + p.col,
                };
                if (lk.maze.get(next.row, next.col) & maze::PATH_BIT) == 0
                    || (lk.maze.get(next.row, next.col) & rgb::MEASURED) != 0
                {
                    continue;
                }
                *lk.maze.get_mut(next.row, next.col) |= rgb::MEASURED;
                lk.map.distances.insert(next, cur.1 + 1);
                bfs.push_back((next, cur.1 + 1));
            }
        }
        start
    } else {
        print::maze_panic!("Thread panic.");
    };

    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    let mut handles = Vec::with_capacity(rgb::NUM_PAINTERS - 1);
    for painter in 1..rgb::NUM_PAINTERS {
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
            bias: 0,
            color_i: rand_color_choice,
            cache: rgb::MEASURED_MASKS[0],
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
                let dist = lk
                    .map
                    .distances
                    .get(&cur)
                    .expect("Could not find map entry?");
                let before = lk.maze.get(cur.row, cur.col);
                if !rgb::has_paint_vals(before) {
                    let intensity = (lk.map.max - dist) as f64 / lk.map.max as f64;
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
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.lock() {
                Err(p) => print::maze_panic!("Panic with lock: {}", p),
                Ok(mut lk) => {
                    let nxt = lk.maze.get(next.row, next.col);
                    let seen = (nxt & guide.cache) != 0;
                    let is_path = maze::is_path(nxt);
                    if !seen && is_path {
                        *lk.maze.get_mut(next.row, next.col) |= guide.cache;
                    }
                    !seen && is_path
                }
            } {
                bfs.push_back(next);
            }
            i = (i + 1) % rgb::NUM_PAINTERS;
            i != guide.bias
        } {}
    }
}
