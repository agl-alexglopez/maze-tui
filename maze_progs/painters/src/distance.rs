use crate::rgb;
use builders::build;
use maze;
use std::collections::{HashSet, VecDeque};

use std::{thread, time};

use rand::{thread_rng, Rng};

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
    painter(&lk.maze, &map);
}

pub fn animate_distance_from_center(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    if monitor.exit() {
        return;
    }
    if monitor
        .solver
        .lock()
        .unwrap_or_else(|_| print::maze_panic!("Thread panicked"))
        .maze
        .is_mini()
    {
        animate_mini_distance_from_center(monitor, speed);
        return;
    }
    let start = if let Ok(mut lk) = monitor.solver.lock() {
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
    let mut handles = Vec::with_capacity(rgb::NUM_PAINTERS);
    let animation = rgb::ANIMATION_SPEEDS[speed as usize];
    for painter in 0..rgb::NUM_PAINTERS {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            painter_animated(
                monitor_clone,
                rgb::ThreadGuide {
                    bias: painter,
                    color_i: rand_color_choice,
                    cache: rgb::MEASURED_MASKS[painter],
                    p: start,
                },
                animation,
            );
        }));
    }
    for h in handles {
        h.join().expect("Error joining a thread.");
    }
}

fn animate_mini_distance_from_center(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let start = if let Ok(mut lk) = monitor.solver.lock() {
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
            if monitor.exit() {
                return;
            }
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
    let mut handles = Vec::with_capacity(rgb::NUM_PAINTERS);
    let animation = rgb::ANIMATION_SPEEDS[speed as usize];
    for painter in 0..rgb::NUM_PAINTERS - 1 {
        let monitor_clone = monitor.clone();
        handles.push(thread::spawn(move || {
            painter_mini_animated(
                monitor_clone,
                rgb::ThreadGuide {
                    bias: painter,
                    color_i: rand_color_choice,
                    cache: rgb::MEASURED_MASKS[painter],
                    p: start,
                },
                animation,
            )
        }));
    }
    painter_mini_animated(
        monitor.clone(),
        rgb::ThreadGuide {
            bias: rgb::NUM_PAINTERS - 1,
            color_i: rand_color_choice,
            cache: rgb::MEASURED_MASKS[0],
            p: start,
        },
        animation,
    );
    for h in handles {
        h.join().expect("Error joining a thread.");
    }
}

// Private Helper Functions-----------------------------------------------------------------------

fn painter(maze: &maze::Maze, map: &monitor::MaxMap) {
    let mut rng = thread_rng();
    let rand_color_choice: usize = rng.gen_range(0..3);
    if maze.style_index() == (maze::MazeStyle::Mini as usize) {
        for r in 0..maze.rows() {
            for c in 0..maze.cols() {
                let cur = maze::Point { row: r, col: c };
                if let Some(dist) = map.distances.get(&cur) {
                    let intensity = (map.max - dist) as f64 / map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut channels: rgb::Rgb = [dark, dark, dark];
                    channels[rand_color_choice] = bright;
                    if cur.row % 2 == 0 {
                        if (maze.get(cur.row + 1, cur.col) & maze::PATH_BIT) != 0 {
                            let neighbor = maze::Point {
                                row: cur.row + 1,
                                col: cur.col,
                            };
                            let neighbor_dist = map.distances.get(&neighbor).expect("Empty map");
                            let intensity = (map.max - neighbor_dist) as f64 / map.max as f64;
                            let dark = (255f64 * intensity) as u8;
                            let bright = 128 + (127f64 * intensity) as u8;
                            let mut neighbor_channels: rgb::Rgb = [dark, dark, dark];
                            neighbor_channels[rand_color_choice] = bright;
                            rgb::print_mini_rgb(
                                Some(channels),
                                Some(neighbor_channels),
                                cur,
                                maze.offset(),
                            );
                        } else {
                            rgb::print_mini_rgb(Some(channels), None, cur, maze.offset());
                        }
                    // This is an odd row.
                    } else if (maze.get(cur.row - 1, cur.col) & maze::PATH_BIT) != 0 {
                        let neighbor = maze::Point {
                            row: cur.row - 1,
                            col: cur.col,
                        };
                        let neighbor_dist = map.distances.get(&neighbor).expect("Empty map");
                        let intensity = (map.max - neighbor_dist) as f64 / map.max as f64;
                        let dark = (255f64 * intensity) as u8;
                        let bright = 128 + (127f64 * intensity) as u8;
                        let mut neighbor_channels: rgb::Rgb = [dark, dark, dark];
                        neighbor_channels[rand_color_choice] = bright;
                        rgb::print_mini_rgb(
                            Some(neighbor_channels),
                            Some(channels),
                            cur,
                            maze.offset(),
                        );
                    } else {
                        rgb::print_mini_rgb(None, Some(channels), cur, maze.offset());
                    }
                } else {
                    build::print_mini_coordinate(maze, cur);
                }
            }
        }
    } else {
        for r in 0..maze.rows() {
            for c in 0..maze.cols() {
                let cur = maze::Point { row: r, col: c };
                match map.distances.get(&cur) {
                    Some(dist) => {
                        let intensity = (map.max - dist) as f64 / map.max as f64;
                        let dark = (255f64 * intensity) as u8;
                        let bright = 128 + (127f64 * intensity) as u8;
                        let mut channels: rgb::Rgb = [dark, dark, dark];
                        channels[rand_color_choice] = bright;
                        rgb::print_rgb(channels, cur, maze.offset());
                    }
                    None => build::print_square(maze, cur),
                }
            }
        }
    }
    print::flush();
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
            i != guide.bias
        } {}
    }
}

fn painter_animated(
    monitor: monitor::MazeReceiver,
    guide: rgb::ThreadGuide,
    animation: rgb::SpeedUnit,
) {
    let mut seen = HashSet::from([guide.p]);
    let mut bfs = VecDeque::from([guide.p]);
    while let Some(cur) = bfs.pop_front() {
        if monitor.exit() {
            return;
        }
        match monitor.solver.lock() {
            Ok(mut lk) => {
                if lk.count == lk.map.distances.len() {
                    return;
                }
                let dist = lk
                    .map
                    .distances
                    .get(&cur)
                    .expect("Could not find map entry?");
                if (lk.maze.get(cur.row, cur.col) & rgb::PAINT) == 0 {
                    let intensity = (lk.map.max - dist) as f64 / lk.map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut channels: rgb::Rgb = [dark, dark, dark];
                    channels[guide.color_i] = bright;
                    rgb::animate_rgb(channels, cur, lk.maze.offset());
                    *lk.maze.get_mut(cur.row, cur.col) |= rgb::PAINT;
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
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Panic with lock: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !seen.contains(&next)
            {
                seen.insert(next);
                bfs.push_back(next);
            }
            i = (i + 1) % rgb::NUM_PAINTERS;
            i != guide.bias
        } {}
    }
}

fn painter_mini_animated(
    monitor: monitor::MazeReceiver,
    guide: rgb::ThreadGuide,
    animation: rgb::SpeedUnit,
) {
    let mut seen = HashSet::from([guide.p]);
    let mut bfs = VecDeque::from([guide.p]);
    while let Some(cur) = bfs.pop_front() {
        if monitor.exit() {
            return;
        }
        match monitor.solver.lock() {
            Ok(mut lk) => {
                if lk.count == lk.map.distances.len() {
                    return;
                }
                if (lk.maze.get(cur.row, cur.col) & rgb::PAINT) == 0 {
                    let dist = lk
                        .map
                        .distances
                        .get(&cur)
                        .expect("Could not find map entry?");
                    let intensity = (lk.map.max - dist) as f64 / lk.map.max as f64;
                    let dark = (255f64 * intensity) as u8;
                    let bright = 128 + (127f64 * intensity) as u8;
                    let mut channels: rgb::Rgb = [dark, dark, dark];
                    channels[guide.color_i] = bright;
                    *lk.maze.get_mut(cur.row, cur.col) |= rgb::PAINT;
                    lk.count += 1;
                    if cur.row % 2 == 0 {
                        if (lk.maze.get(cur.row + 1, cur.col) & maze::PATH_BIT) != 0 {
                            let neighbor = maze::Point {
                                row: cur.row + 1,
                                col: cur.col,
                            };
                            *lk.maze.get_mut(neighbor.row, neighbor.col) |= rgb::PAINT;
                            let neighbor_dist = lk.map.distances.get(&neighbor).expect("Empty map");
                            let intensity = (lk.map.max - neighbor_dist) as f64 / lk.map.max as f64;
                            let dark = (255f64 * intensity) as u8;
                            let bright = 128 + (127f64 * intensity) as u8;
                            let mut neighbor_channels: rgb::Rgb = [dark, dark, dark];
                            neighbor_channels[guide.color_i] = bright;
                            rgb::animate_mini_rgb(
                                Some(channels),
                                Some(neighbor_channels),
                                cur,
                                lk.maze.offset(),
                            );
                        } else {
                            rgb::animate_mini_rgb(Some(channels), None, cur, lk.maze.offset());
                        }
                    // This is an odd row.
                    } else if (lk.maze.get(cur.row - 1, cur.col) & maze::PATH_BIT) != 0 {
                        let neighbor = maze::Point {
                            row: cur.row - 1,
                            col: cur.col,
                        };
                        *lk.maze.get_mut(neighbor.row, neighbor.col) |= rgb::PAINT;
                        let neighbor_dist = lk.map.distances.get(&neighbor).expect("Empty map");
                        let intensity = (lk.map.max - neighbor_dist) as f64 / lk.map.max as f64;
                        let dark = (255f64 * intensity) as u8;
                        let bright = 128 + (127f64 * intensity) as u8;
                        let mut neighbor_channels: rgb::Rgb = [dark, dark, dark];
                        neighbor_channels[guide.color_i] = bright;
                        rgb::animate_mini_rgb(
                            Some(neighbor_channels),
                            Some(channels),
                            cur,
                            lk.maze.offset(),
                        );
                    } else {
                        rgb::animate_mini_rgb(None, Some(channels), cur, lk.maze.offset());
                    }
                }
            }
            Err(p) => print::maze_panic!("Thread panicked with lock: {}", p),
        }

        thread::sleep(time::Duration::from_micros(animation));
        let mut i = guide.bias;
        while {
            let p = &maze::CARDINAL_DIRECTIONS[i];
            let next = maze::Point {
                row: cur.row + p.row,
                col: cur.col + p.col,
            };
            if match monitor.solver.lock() {
                Err(p) => print::maze_panic!("Panic with lock: {}", p),
                Ok(lk) => (lk.maze.get(next.row, next.col) & maze::PATH_BIT) != 0,
            } && !seen.contains(&next)
            {
                seen.insert(next);
                bfs.push_back(next);
            }
            i = (i + 1) % rgb::NUM_PAINTERS;
            i != guide.bias
        } {}
    }
}
