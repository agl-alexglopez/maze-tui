use crate::build;
use maze;
use speed;

pub fn add_cross(maze: &mut maze::Maze) {
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if (r == maze.row_size() / 2 && c > 1 && c < maze.col_size() - 2)
                || (c == maze.col_size() / 2 && r > 1 && r < maze.row_size() - 2)
            {
                build::build_path(maze, maze::Point { row: r, col: c });
                if c + 1 < maze.col_size() - 2 {
                    build::build_path(maze, maze::Point { row: r, col: c + 1 });
                }
            }
        }
    }
}

pub fn add_cross_animated(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if maze.exit() {
                return;
            }
            if (r == maze.row_size() / 2 && c > 1 && c < maze.col_size() - 2)
                || (c == maze.col_size() / 2 && r > 1 && r < maze.row_size() - 2)
            {
                build::build_path_animated(maze, maze::Point { row: r, col: c }, animation);
                if c + 1 < maze.col_size() - 2 {
                    build::build_path_animated(maze, maze::Point { row: r, col: c + 1 }, animation);
                }
            }
        }
    }
}

pub fn add_mini_cross_animated(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    for r in 0..maze.row_size() {
        for c in 0..maze.col_size() {
            if maze.exit() {
                return;
            }
            if (r == maze.row_size() / 2 && c > 1 && c < maze.col_size() - 2)
                || (c == maze.col_size() / 2 && r > 1 && r < maze.row_size() - 2)
            {
                build::build_mini_path_animated(maze, maze::Point { row: r, col: c }, animation);
                if c + 1 < maze.col_size() - 2 {
                    build::build_mini_path_animated(
                        maze,
                        maze::Point { row: r, col: c + 1 },
                        animation,
                    );
                }
            }
        }
    }
}

pub fn add_x(maze: &mut maze::Maze) {
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            add_positive_slope(maze, maze::Point { row: r, col: c });
            add_negative_slope(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn add_x_animated(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            if maze.exit() {
                return;
            }
            add_positive_slope_animated(maze, maze::Point { row: r, col: c }, animation);
            add_negative_slope_animated(maze, maze::Point { row: r, col: c }, animation);
        }
    }
}

pub fn add_mini_x_animated(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            if maze.exit() {
                return;
            }
            add_mini_positive_slope_animated(maze, maze::Point { row: r, col: c }, animation);
            add_mini_negative_slope_animated(maze, maze::Point { row: r, col: c }, animation);
        }
    }
}

fn add_positive_slope(maze: &mut maze::Maze, p: maze::Point) {
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.col_size() - 2 && p.col > 1 {
        build::build_path(maze, p);
        if p.col + 1 < maze.col_size() - 2 {
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
        if p.col + 2 < maze.col_size() - 2 {
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
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.col_size() - 2 && p.col > 1 {
        build::build_path_animated(maze, p, speed);
        if p.col + 1 < maze.col_size() - 2 {
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
        if p.col + 2 < maze.col_size() - 2 {
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
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ((cur_row - b) / slope) as i32;
    if p.col == on_slope && p.col < maze.col_size() - 2 && p.col > 1 {
        build::build_mini_path_animated(maze, p, speed);
        if p.col + 1 < maze.col_size() - 2 {
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
        if p.col + 2 < maze.col_size() - 2 {
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
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.col_size() - 2 && p.row < maze.row_size() - 2 {
        build::build_path(maze, p);
        if p.col + 1 < maze.col_size() - 2 {
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
        if p.col + 2 < maze.col_size() - 2 {
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
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.col_size() - 2 && p.row < maze.row_size() - 2 {
        build::build_path_animated(maze, p, speed);
        if p.col + 1 < maze.col_size() - 2 {
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
        if p.col + 2 < maze.col_size() - 2 {
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
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (col_size - 2.0f32);
    let b = row_size - (2.0f32 * slope);
    let on_line = ((cur_row - b) / slope) as i32;
    if p.col == on_line && p.col > 1 && p.col < maze.col_size() - 2 && p.row < maze.row_size() - 2 {
        build::build_mini_path_animated(maze, p, speed);
        if p.col + 1 < maze.col_size() - 2 {
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
        if p.col + 2 < maze.col_size() - 2 {
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
