use std::io;

use crate::build;
use crossterm::{
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use maze;
use rand::{seq::SliceRandom, thread_rng, Rng};
use speed;
use std::{thread, time};

type DirectionMarker = build::BacktrackMarker;

const CARVER: char = '█';
// Same color for each looks good for now change for style.
const CARVING: u8 = 9;
const HUNTING: u8 = 9;
const WALL_UP_DOWN_RIGHT: maze::Square = 0b0111;
const WALL_UP_DOWN_LEFT: maze::Square = 0b1101;
const WALL_LEFT_RIGHT: maze::Square = 0b1010;
const GOING_NORTH: DirectionMarker = build::FROM_NORTH;
const GOING_EAST: DirectionMarker = build::FROM_EAST;
const GOING_SOUTH: DirectionMarker = build::FROM_SOUTH;
const GOING_WEST: DirectionMarker = build::FROM_WEST;

///
/// Data only maze generator
///

pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.rows() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.cols() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; 4] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    let mut highest_completed_row = 1;
    'carving: loop {
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                build::join_squares(&mut lk.maze, cur, branch);
                cur = branch;
                continue 'carving;
            }
        }

        let mut set_highest_completed_row = false;
        for r in (highest_completed_row..lk.maze.rows() - 1).step_by(2) {
            for c in (1..lk.maze.cols() - 1).step_by(2) {
                let start_candidate = maze::Point { row: r, col: c };
                if (lk.maze.get(r, c) & build::BUILDER_BIT) == 0 {
                    if !set_highest_completed_row {
                        highest_completed_row = r;
                        set_highest_completed_row = true;
                    }
                    for dir in &build::GENERATE_DIRECTIONS {
                        let next = maze::Point {
                            row: r + dir.row,
                            col: c + dir.col,
                        };
                        if build::is_square_within_perimeter_walls(&lk.maze, next)
                            && build::is_built(lk.maze.get(next.row, next.col))
                        {
                            build::join_squares(&mut lk.maze, start_candidate, next);
                            cur = start_candidate;
                            continue 'carving;
                        }
                    }
                }
            }
        }
        return;
    }
}

///
/// History based generator for animation and playback.
///

pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_history_with_walls(&mut lk.maze);
    let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.rows() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.cols() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; 4] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    let mut highest_completed_row = 1;
    hunter_laser_history(&mut lk.maze, highest_completed_row);
    'carving: loop {
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                carve_forward_history(&mut lk.maze, cur, branch, highest_completed_row + 1);
                cur = branch;
                continue 'carving;
            }
        }
        let mut set_highest_completed_row = false;
        for r in (highest_completed_row..lk.maze.rows() - 1).step_by(2) {
            for c in (1..lk.maze.cols() - 1).step_by(2) {
                let start_candidate = maze::Point { row: r, col: c };
                if (lk.maze.get(r, c) & build::BUILDER_BIT) == 0 {
                    if !set_highest_completed_row {
                        if r != highest_completed_row {
                            reset_hunter_laser_history(&mut lk.maze, highest_completed_row);
                            hunter_laser_history(&mut lk.maze, r);
                        }
                        highest_completed_row = r;
                        set_highest_completed_row = true;
                    }
                    for dir in &build::GENERATE_DIRECTIONS {
                        let next = maze::Point {
                            row: r + dir.row,
                            col: c + dir.col,
                        };
                        if build::is_square_within_perimeter_walls(&lk.maze, next)
                            && build::is_built(lk.maze.get(next.row, next.col))
                        {
                            carve_forward_history(
                                &mut lk.maze,
                                start_candidate,
                                next,
                                highest_completed_row + 1,
                            );
                            cur = start_candidate;
                            continue 'carving;
                        }
                    }
                }
            }
        }
        reset_hunter_laser_history(&mut lk.maze, highest_completed_row);
        return;
    }
}

fn carve_forward_history(maze: &mut maze::Maze, cur: maze::Point, next: maze::Point, min_row: i32) {
    let mut wall: maze::Point = cur;
    let direction = if next.row < cur.row {
        wall.row -= 1;
        GOING_NORTH
    } else if next.row > cur.row {
        wall.row += 1;
        GOING_SOUTH
    } else if next.col < cur.col {
        wall.col -= 1;
        GOING_WEST
    } else if next.col > cur.col {
        wall.col += 1;
        GOING_EAST
    } else {
        print::maze_panic!("Wall break error. Cur: {:?} Next: {:?}", cur, next);
    };
    carve_forward_wall_history(maze, cur, direction, min_row);
    carve_forward_wall_history(maze, wall, direction, min_row);
    carve_forward_wall_history(maze, next, direction, min_row);
}

fn carve_forward_wall_history(
    maze: &mut maze::Maze,
    p: maze::Point,
    direction: DirectionMarker,
    min_row: i32,
) {
    let mut wall_changes = [maze::Delta::default(); 5];
    let mut burst = 1;
    let before = maze.get(p.row, p.col);
    let mut after =
        ((before & !build::MARKERS_MASK) & !maze::WALL_MASK) | maze::PATH_BIT | build::BUILDER_BIT;
    *maze.get_mut(p.row, p.col) =
        ((before & !build::MARKERS_MASK) & !maze::WALL_MASK) | maze::PATH_BIT | build::BUILDER_BIT;
    if p.row > min_row {
        after |= direction;
        *maze.get_mut(p.row, p.col) |= direction;
    }
    wall_changes[0] = maze::Delta {
        id: p,
        before,
        after,
        burst,
    };
    if p.row > 0 {
        let square = maze.get(p.row - 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row - 1,
                col: p.col,
            },
            before: square,
            after: square & !maze::SOUTH_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row - 1, p.col) &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.rows() {
        let square = maze.get(p.row + 1, p.col);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row + 1,
                col: p.col,
            },
            before: square,
            after: square & !maze::NORTH_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row + 1, p.col) &= !maze::NORTH_WALL;
    }
    if p.col > 0 {
        let square = maze.get(p.row, p.col - 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col - 1,
            },
            before: square,
            after: square & !maze::EAST_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col - 1) &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.cols() {
        let square = maze.get(p.row, p.col + 1);
        wall_changes[burst] = maze::Delta {
            id: maze::Point {
                row: p.row,
                col: p.col + 1,
            },
            before: square,
            after: square & !maze::WEST_WALL,
            burst: 1,
        };
        burst += 1;
        *maze.get_mut(p.row, p.col + 1) &= !maze::WEST_WALL;
    }
    wall_changes[0].burst = burst;
    wall_changes[burst - 1].burst = burst;
    maze.build_history.push_burst(&wall_changes[0..burst]);
}

fn hunter_laser_history(maze: &mut maze::Maze, current_row: i32) {
    let mut delta_vec = Vec::new();
    for c in 0..maze.cols() {
        let square = maze.get(current_row, c);
        delta_vec.push(maze::Delta {
            id: maze::Point {
                row: current_row,
                col: c,
            },
            before: square,
            after: (square & !build::MARKERS_MASK) | build::FROM_SOUTH,
            burst: 1,
        });
        *maze.get_mut(current_row, c) = (square & !build::MARKERS_MASK) | build::FROM_SOUTH;
    }
    delta_vec[0].burst = maze.cols() as usize;
    delta_vec[(maze.cols() - 1) as usize].burst = maze.cols() as usize;
    maze.build_history.push_burst(delta_vec.as_slice());
}

fn reset_hunter_laser_history(maze: &mut maze::Maze, current_row: i32) {
    if current_row != maze.rows() - 1 {
        let mut delta_vec = Vec::with_capacity((maze.cols() * 2) as usize);
        for c in 0..maze.cols() {
            let square = maze.get(current_row, c);
            delta_vec.push(maze::Delta {
                id: maze::Point {
                    row: current_row,
                    col: c,
                },
                before: square,
                after: square & !build::MARKERS_MASK,
                burst: 1,
            });
            *maze.get_mut(current_row, c) &= !build::MARKERS_MASK;
            let next_square = maze.get(current_row + 1, c);
            delta_vec.push(maze::Delta {
                id: maze::Point {
                    row: current_row + 1,
                    col: c,
                },
                before: next_square,
                after: next_square & !build::MARKERS_MASK,
                burst: 1,
            });
            *maze.get_mut(current_row + 1, c) &= !build::MARKERS_MASK;
        }
        delta_vec[0].burst = (maze.cols() * 2) as usize;
        delta_vec[(maze.cols() * 2 - 1) as usize].burst = (maze.cols() * 2) as usize;
        maze.build_history.push_burst(delta_vec.as_slice());
        return;
    }
    let mut delta_vec = Vec::with_capacity(maze.cols() as usize);
    for c in 0..maze.cols() {
        let square = maze.get(current_row, c);
        delta_vec.push(maze::Delta {
            id: maze::Point {
                row: current_row,
                col: c,
            },
            before: square,
            after: square & !build::MARKERS_MASK,
            burst: 1,
        });
        *maze.get_mut(current_row, c) &= !build::MARKERS_MASK;
    }
    delta_vec[0].burst = maze.cols() as usize;
    delta_vec[(maze.cols() - 1) as usize].burst = maze.cols() as usize;
    maze.build_history.push_burst(delta_vec.as_slice());
}

///
/// Cursor based generator.
///

pub fn animate_maze(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    if lk.maze.is_mini() {
        drop(lk);
        animate_mini_maze(monitor, speed);
        return;
    }
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
        let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.rows() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.cols() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; 4] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    let mut highest_completed_row = 1;
    'carving: loop {
        if monitor.exit() {
            return;
        }
        print::set_cursor_position(cur, lk.maze.offset());
        execute!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(CARVING)),
            Print(CARVER),
            ResetColor
        )
        .expect("Printer broke");
        thread::sleep(time::Duration::from_micros(animation));
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                build::join_squares_animated(&mut lk.maze, cur, branch, animation);
                print::set_cursor_position(branch, lk.maze.offset());
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::AnsiValue(CARVING)),
                    Print(CARVER),
                    ResetColor
                )
                .expect("Printer broke");
                thread::sleep(time::Duration::from_micros(animation));
                cur = branch;
                continue 'carving;
            }
        }

        let mut set_highest_completed_row = false;
        for r in (highest_completed_row..lk.maze.rows() - 1).step_by(2) {
            shoot_hunter_laser(&lk.maze, r);
            for c in (1..lk.maze.cols() - 1).step_by(2) {
                let start_candidate = maze::Point { row: r, col: c };
                if (lk.maze.get(r, c) & build::BUILDER_BIT) == 0 {
                    if !set_highest_completed_row {
                        highest_completed_row = r;
                        set_highest_completed_row = true;
                    }
                    for dir in &build::GENERATE_DIRECTIONS {
                        let next = maze::Point {
                            row: r + dir.row,
                            col: c + dir.col,
                        };
                        if build::is_square_within_perimeter_walls(&lk.maze, next)
                            && build::is_built(lk.maze.get(next.row, next.col))
                        {
                            build::join_squares_animated(
                                &mut lk.maze,
                                start_candidate,
                                next,
                                animation,
                            );
                            cur = start_candidate;
                            continue 'carving;
                        }
                    }
                }
            }
            reset_row(&lk.maze, r);
        }
        return;
    }
}

pub fn animate_mini_maze(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation: build::SpeedUnit = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
        let mut gen = thread_rng();
    let start: maze::Point = maze::Point {
        row: 2 * (gen.gen_range(1..lk.maze.rows() - 2) / 2) + 1,
        col: 2 * (gen.gen_range(1..lk.maze.cols() - 2) / 2) + 1,
    };
    let mut random_direction_indices: [usize; 4] = [0, 1, 2, 3];
    let mut cur: maze::Point = start;
    let mut highest_completed_row = 1;
    'carving: loop {
        if monitor.exit() {
            return;
        }
        flush_mini_carver(&lk.maze, cur, animation);
        random_direction_indices.shuffle(&mut gen);
        for &i in random_direction_indices.iter() {
            let direction = &build::GENERATE_DIRECTIONS[i];
            let branch = maze::Point {
                row: cur.row + direction.row,
                col: cur.col + direction.col,
            };
            if build::can_build_new_square(&lk.maze, branch) {
                build::join_minis_animated(&mut lk.maze, cur, branch, animation);
                print::set_cursor_position(
                    maze::Point {
                        row: branch.row / 2,
                        col: branch.col,
                    },
                    lk.maze.offset(),
                );
                flush_mini_carver(&lk.maze, branch, animation);
                cur = branch;
                continue 'carving;
            }
        }

        let mut set_highest_completed_row = false;
        for r in (highest_completed_row..lk.maze.rows() - 1).step_by(2) {
            shoot_mini_laser(&lk.maze, r);
            for c in (1..lk.maze.cols() - 1).step_by(2) {
                let start_candidate = maze::Point { row: r, col: c };
                if (lk.maze.get(r, c) & build::BUILDER_BIT) == 0 {
                    if !set_highest_completed_row {
                        highest_completed_row = r;
                        set_highest_completed_row = true;
                    }
                    for dir in &build::GENERATE_DIRECTIONS {
                        let next = maze::Point {
                            row: r + dir.row,
                            col: c + dir.col,
                        };
                        if build::is_square_within_perimeter_walls(&lk.maze, next)
                            && build::is_built(lk.maze.get(next.row, next.col))
                        {
                            build::join_minis_animated(
                                &mut lk.maze,
                                start_candidate,
                                next,
                                animation,
                            );
                            cur = start_candidate;
                            continue 'carving;
                        }
                    }
                }
            }
            reset_mini_row(&lk.maze, r);
        }
        return;
    }
}

fn shoot_hunter_laser(maze: &maze::Maze, current_row: i32) {
    let mut stdout = io::stdout();
    print::set_cursor_position(
        maze::Point {
            row: current_row,
            col: 0,
        },
        maze.offset(),
    );
    // Don't reset the color until the end.
    queue!(
        stdout,
        SetForegroundColor(Color::AnsiValue(HUNTING)),
        Print(maze.wall_char(WALL_UP_DOWN_RIGHT)),
    )
    .expect("Printer broke");
    for c in 1..maze.cols() - 1 {
        print::set_cursor_position(
            maze::Point {
                row: current_row,
                col: c,
            },
            maze.offset(),
        );
        queue!(stdout, Print(maze.wall_char(WALL_LEFT_RIGHT)),).expect("Printer broke");
    }
    print::set_cursor_position(
        maze::Point {
            row: current_row,
            col: maze.cols() - 1,
        },
        maze.offset(),
    );
    // Color is now reset.
    queue!(
        stdout,
        SetForegroundColor(Color::AnsiValue(HUNTING)),
        Print(maze.wall_char(WALL_UP_DOWN_LEFT)),
        ResetColor
    )
    .expect("Printer broke");
    print::flush();
}

fn shoot_mini_laser(maze: &maze::Maze, current_row: i32) {
    let mut stdout = io::stdout();
    for c in 0..maze.cols() {
        print::set_cursor_position(
            maze::Point {
                row: current_row / 2,
                col: c,
            },
            maze.offset(),
        );
        if maze.wall_at(current_row + 1, c) {
            queue!(
                stdout,
                SetBackgroundColor(Color::AnsiValue(HUNTING)),
                Print('▄'),
                ResetColor
            )
            .expect("Printer broke");
        } else {
            queue!(
                stdout,
                SetBackgroundColor(Color::AnsiValue(HUNTING)),
                SetForegroundColor(Color::Black),
                Print('▄'),
                ResetColor
            )
            .expect("Printer broke");
        }
    }
    print::flush();
}

fn reset_row(maze: &maze::Maze, current_row: i32) {
    for c in 0..maze.cols() {
        build::print_square(
            maze,
            maze::Point {
                row: current_row,
                col: c,
            },
        );
    }
    print::flush();
}

fn reset_mini_row(maze: &maze::Maze, current_row: i32) {
    for c in 0..maze.cols() {
        build::print_mini_coordinate(
            maze,
            maze::Point {
                row: current_row,
                col: c,
            },
        );
    }
    print::flush();
}

fn flush_mini_carver(maze: &maze::Maze, p: maze::Point, animation: build::SpeedUnit) {
    print::set_cursor_position(
        maze::Point {
            row: p.row / 2,
            col: p.col,
        },
        maze.offset(),
    );
    if maze.wall_at(p.row - 1, p.col) {
        execute!(
            io::stdout(),
            SetBackgroundColor(Color::AnsiValue(CARVING)),
            Print('▀'),
            ResetColor
        )
        .expect("Printer broke");
        thread::sleep(time::Duration::from_micros(animation));
    } else {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::AnsiValue(CARVING)),
            Print('▄'),
            ResetColor
        )
        .expect("Printer broke");
        thread::sleep(time::Duration::from_micros(animation));
    }
}
