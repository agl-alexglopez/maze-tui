use crate::build;
use maze;
use speed;

pub fn add_cross_history(monitor: monitor::MazeMonitor) {
    todo!();
}

pub fn add_cross(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    for r in 0..lk.maze.rows() {
        for c in 0..lk.maze.cols() {
            if (r == lk.maze.rows() / 2 && c > 1 && c < lk.maze.cols() - 2)
                || (c == lk.maze.cols() / 2 && r > 1 && r < lk.maze.rows() - 2)
            {
                build::build_path(&mut lk.maze, maze::Point { row: r, col: c });
                if c + 1 < lk.maze.cols() - 2 {
                    build::build_path(&mut lk.maze, maze::Point { row: r, col: c + 1 });
                }
            }
        }
    }
}

pub fn add_cross_animated(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    if lk.maze.is_mini() {
        drop(lk);
        add_mini_cross_animated(monitor, speed);
        return;
    }
    let animation = build::BUILDER_SPEEDS[speed as usize];
    for r in 0..lk.maze.rows() {
        for c in 0..lk.maze.cols() {
            if monitor.exit() {
                return;
            }
            if (r == lk.maze.rows() / 2 && c > 1 && c < lk.maze.cols() - 2)
                || (c == lk.maze.cols() / 2 && r > 1 && r < lk.maze.rows() - 2)
            {
                build::build_path_animated(&mut lk.maze, maze::Point { row: r, col: c }, animation);
                if c + 1 < lk.maze.cols() - 2 {
                    build::build_path_animated(
                        &mut lk.maze,
                        maze::Point { row: r, col: c + 1 },
                        animation,
                    );
                }
            }
        }
    }
}

fn add_mini_cross_animated(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation = build::BUILDER_SPEEDS[speed as usize];
    for r in 0..lk.maze.rows() {
        for c in 0..lk.maze.cols() {
            if monitor.exit() {
                return;
            }
            if (r == lk.maze.rows() / 2 && c > 1 && c < lk.maze.cols() - 2)
                || (c == lk.maze.cols() / 2 && r > 1 && r < lk.maze.rows() - 2)
            {
                build::build_mini_path_animated(
                    &mut lk.maze,
                    maze::Point { row: r, col: c },
                    animation,
                );
                if c + 1 < lk.maze.cols() - 2 {
                    build::build_mini_path_animated(
                        &mut lk.maze,
                        maze::Point { row: r, col: c + 1 },
                        animation,
                    );
                }
            }
        }
    }
}

pub fn add_x_history(monitor: monitor::MazeMonitor) {
    todo!();
}

pub fn add_x(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    for r in 1..lk.maze.rows() - 1 {
        for c in 1..lk.maze.cols() - 1 {
            add_positive_slope(&mut lk.maze, maze::Point { row: r, col: c });
            add_negative_slope(&mut lk.maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn add_x_animated(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    if lk.maze.is_mini() {
        drop(lk);
        add_mini_x_animated(monitor, speed);
        return;
    }
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    for r in 1..lk.maze.rows() - 1 {
        for c in 1..lk.maze.cols() - 1 {
            if monitor.exit() {
                return;
            }
            add_positive_slope_animated(&mut lk.maze, maze::Point { row: r, col: c }, animation);
            add_negative_slope_animated(&mut lk.maze, maze::Point { row: r, col: c }, animation);
        }
    }
}

fn add_mini_x_animated(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    for r in 1..lk.maze.rows() - 1 {
        for c in 1..lk.maze.cols() - 1 {
            if monitor.exit() {
                return;
            }
            add_mini_positive_slope_animated(
                &mut lk.maze,
                maze::Point { row: r, col: c },
                animation,
            );
            add_mini_negative_slope_animated(
                &mut lk.maze,
                maze::Point { row: r, col: c },
                animation,
            );
        }
    }
}

fn add_positive_slope(maze: &mut maze::Maze, p: maze::Point) {
    let row_size = maze.rows() as f32 - 2.0f32;
    let col_size = maze.cols() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.cols() - 2 && p.col > 1 {
        build::build_path(maze, p);
        if p.col + 1 < maze.cols() - 2 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
            );
        }
        if p.col - 1 > 1 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
            );
        }
        if p.col + 2 < maze.cols() - 2 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
            );
        }
        if p.col - 2 > 1 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
            );
        }
    }
}

fn add_positive_slope_animated(maze: &mut maze::Maze, p: maze::Point, speed: build::SpeedUnit) {
    let row_size = maze.rows() as f32 - 2.0f32;
    let col_size = maze.cols() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.cols() - 2 && p.col > 1 {
        build::build_path_animated(maze, p, speed);
        if p.col + 1 < maze.cols() - 2 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
                speed,
            );
        }
        if p.col - 1 > 1 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
                speed,
            );
        }
        if p.col + 2 < maze.cols() - 2 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
                speed,
            );
        }
        if p.col - 2 > 1 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
                speed,
            );
        }
    }
}

fn add_mini_positive_slope_animated(
    maze: &mut maze::Maze,
    p: maze::Point,
    speed: build::SpeedUnit,
) {
    let row_size = maze.rows() as f32 - 2.0f32;
    let col_size = maze.cols() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.cols() - 2 && p.col > 1 {
        build::build_mini_path_animated(maze, p, speed);
        if p.col + 1 < maze.cols() - 2 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
                speed,
            );
        }
        if p.col - 1 > 1 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
                speed,
            );
        }
        if p.col + 2 < maze.cols() - 2 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
                speed,
            );
        }
        if p.col - 2 > 1 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
                speed,
            );
        }
    }
}

fn add_negative_slope(maze: &mut maze::Maze, p: maze::Point) {
    let row_size = maze.rows() as f32 - 2.0f32;
    let col_size = maze.cols() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.cols() - 2 && p.row < maze.rows() - 2 {
        build::build_path(maze, p);
        if p.col + 1 < maze.cols() - 2 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
            );
        }
        if p.col - 1 > 1 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
            );
        }
        if p.col + 2 < maze.cols() - 2 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
            );
        }
        if p.col - 2 > 1 {
            build::build_path(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
            );
        }
    }
}

fn add_negative_slope_animated(maze: &mut maze::Maze, p: maze::Point, speed: build::SpeedUnit) {
    let row_size = maze.rows() as f32 - 2.0f32;
    let col_size = maze.cols() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.cols() - 2 && p.row < maze.rows() - 2 {
        build::build_path_animated(maze, p, speed);
        if p.col + 1 < maze.cols() - 2 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
                speed,
            );
        }
        if p.col - 1 > 1 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
                speed,
            );
        }
        if p.col + 2 < maze.cols() - 2 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
                speed,
            );
        }
        if p.col - 2 > 1 {
            build::build_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
                speed,
            );
        }
    }
}

fn add_mini_negative_slope_animated(
    maze: &mut maze::Maze,
    p: maze::Point,
    speed: build::SpeedUnit,
) {
    let row_size = maze.rows() as f32 - 2.0f32;
    let col_size = maze.cols() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.cols() - 2 && p.row < maze.rows() - 2 {
        build::build_mini_path_animated(maze, p, speed);
        if p.col + 1 < maze.cols() - 2 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 1,
                },
                speed,
            );
        }
        if p.col - 1 > 1 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 1,
                },
                speed,
            );
        }
        if p.col + 2 < maze.cols() - 2 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col + 2,
                },
                speed,
            );
        }
        if p.col - 2 > 1 {
            build::build_mini_path_animated(
                maze,
                maze::Point {
                    row: p.row,
                    col: p.col - 2,
                },
                speed,
            );
        }
    }
}
