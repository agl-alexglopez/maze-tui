use crate::rgb;
use builders::build::print_square;
use crossterm::{execute, style::Print};
use maze;
use solvers::solve;
use std::collections::{HashSet, VecDeque};
use std::io::{self};

use std::{thread, time};

use rand::{thread_rng, Rng};

pub fn paint_distance_from_center(monitor: solve::SolverMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("Lock panic."),
    };

    let row_mid = lk.maze.row_size() / 2;
    let col_mid = lk.maze.col_size() / 2;
    let start = maze::Point {
        row: row_mid + 1 - (row_mid % 2),
        col: col_mid + 1 - (col_mid % 2),
    };
    let mut map = solve::MaxMap::new(start, 0);
    let mut bfs = VecDeque::from([(start, 0u64)]);
    lk.maze[start.row as usize][start.col as usize] |= rgb::MEASURE;
    while let Some(cur) = bfs.pop_front() {
        if cur.1 > map.max {
            map.max = cur.1;
        }
        for &p in maze::CARDINAL_DIRECTIONS.iter() {
            let next = maze::Point {
                row: cur.0.row + p.row,
                col: cur.0.col + p.col,
            };
            if (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                || (lk.maze[next.row as usize][next.col as usize] & rgb::MEASURE) != 0
            {
                continue;
            }
            lk.maze[next.row as usize][next.col as usize] |= rgb::MEASURE;
            map.distances.insert(next, cur.1 + 1);
            bfs.push_back((next, cur.1 + 1));
        }
    }
    painter(&mut lk.maze, &map);
    match execute!(io::stdout(), Print('\n')) {
        Ok(_) => {}
        Err(_) => print::maze_panic!("Painter failed to print."),
    }
}

pub fn animate_distance_from_center(monitor: solve::SolverMonitor, speed: speed::Speed) {
    let start = if let Ok(mut lk) = monitor.lock() {
        let row_mid = lk.maze.row_size() / 2;
        let col_mid = lk.maze.col_size() / 2;
        let start = maze::Point {
            row: row_mid + 1 - (row_mid % 2),
            col: col_mid + 1 - (col_mid % 2),
        };
        lk.map.distances.insert(start, 0);
        let mut bfs = VecDeque::from([(start, 0u64)]);
        lk.maze[start.row as usize][start.col as usize] |= rgb::MEASURE;
        while let Some(cur) = bfs.pop_front() {
            if cur.1 > lk.map.max {
                lk.map.max = cur.1;
            }
            for &p in maze::CARDINAL_DIRECTIONS.iter() {
                let next = maze::Point {
                    row: cur.0.row + p.row,
                    col: cur.0.col + p.col,
                };
                if (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) == 0
                    || (lk.maze[next.row as usize][next.col as usize] & rgb::MEASURE) != 0
                {
                    continue;
                }
                lk.maze[next.row as usize][next.col as usize] |= rgb::MEASURE;
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
            print::set_cursor_position(
                maze::Point {
                    row: lk.maze.row_size(),
                    col: lk.maze.col_size(),
                },
                lk.maze.offset(),
            );
            match execute!(io::stdout(), Print('\n')) {
                Ok(_) => {}
                Err(_) => print::maze_panic!("Painter failed to print."),
            }
        }
        Err(p) => print::maze_panic!("Thread panicked: {}", p),
    };
}

// Private Helper Functions-----------------------------------------------------------------------

fn painter(maze: &maze::Maze, map: &solve::MaxMap) {
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
                    let mut channels: rgb::Rgb = [dark, dark, dark];
                    channels[rand_color_choice] = bright;
                    rgb::print_rgb(channels, cur, maze.offset());
                }
                None => print_square(&maze, cur),
            }
        }
    }
    match execute!(io::stdout(), Print('\n')) {
        Ok(_) => {}
        Err(_) => print::maze_panic!("Painter failed to print."),
    }
}

fn painter_animated(
    monitor: solve::SolverMonitor,
    guide: rgb::ThreadGuide,
    animation: rgb::SpeedUnit,
) {
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
                    let mut channels: rgb::Rgb = [dark, dark, dark];
                    channels[guide.color_i] = bright;
                    rgb::animate_rgb(channels, cur, lk.maze.offset());
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
                Ok(lk) => (lk.maze[next.row as usize][next.col as usize] & maze::PATH_BIT) != 0,
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
