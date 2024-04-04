use crate::build;
use maze;

use rand::{rngs::ThreadRng, thread_rng, Rng};

type Height = i32;
type Width = i32;

#[derive(Clone, Copy)]
struct Chamber {
    offset: maze::Point,
    h: Height,
    w: Width,
}

const MIN_CHAMBER: i32 = 3;

///
/// Data only maze generator
///

pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::build_wall_outline(&mut lk.maze);
    let mut rng = thread_rng();
    let mut chamber_stack: Vec<Chamber> = Vec::from([Chamber {
        offset: maze::Point { row: 0, col: 0 },
        h: lk.maze.rows(),
        w: lk.maze.cols(),
    }]);
    while let Some(chamber) = chamber_stack.pop() {
        if chamber.h >= chamber.w && chamber.w > MIN_CHAMBER {
            let divide = random_even_div(&mut rng, chamber.h);
            let passage = rand_odd_pass(&mut rng, chamber.w);
            for c in 0..chamber.w {
                if c == passage {
                    continue;
                }
                build::build_wall_line(
                    &mut lk.maze,
                    maze::Point {
                        row: chamber.offset.row + divide,
                        col: chamber.offset.col + c,
                    },
                );
            }
            chamber_stack.push(Chamber {
                offset: chamber.offset,
                h: divide + 1,
                w: chamber.w,
            });
            chamber_stack.push(Chamber {
                offset: maze::Point {
                    row: chamber.offset.row + divide,
                    col: chamber.offset.col,
                },
                h: chamber.h - divide,
                w: chamber.w,
            });
        } else if chamber.w > chamber.h && chamber.h > MIN_CHAMBER {
            let divide = random_even_div(&mut rng, chamber.w);
            let passage = rand_odd_pass(&mut rng, chamber.h);
            for r in 0..chamber.h {
                if r == passage {
                    continue;
                }
                build::build_wall_line(
                    &mut lk.maze,
                    maze::Point {
                        row: chamber.offset.row + r,
                        col: chamber.offset.col + divide,
                    },
                );
            }
            chamber_stack.push(Chamber {
                offset: chamber.offset,
                h: chamber.h,
                w: divide + 1,
            });
            chamber_stack.push(Chamber {
                offset: maze::Point {
                    row: chamber.offset.row,
                    col: chamber.offset.col + divide,
                },
                h: chamber.h,
                w: chamber.w - divide,
            });
        }
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
    build::build_wall_outline_history(&mut lk.maze);
    let mut rng = thread_rng();
    let mut chamber_stack: Vec<Chamber> = Vec::from([Chamber {
        offset: maze::Point { row: 0, col: 0 },
        h: lk.maze.rows(),
        w: lk.maze.cols(),
    }]);
    while let Some(chamber) = chamber_stack.pop() {
        if chamber.h >= chamber.w && chamber.w > MIN_CHAMBER {
            let divide = random_even_div(&mut rng, chamber.h);
            let passage = rand_odd_pass(&mut rng, chamber.w);
            for c in 0..chamber.w {
                if c == passage {
                    continue;
                }
                build::build_wall_line_history(
                    &mut lk.maze,
                    maze::Point {
                        row: chamber.offset.row + divide,
                        col: chamber.offset.col + c,
                    },
                );
            }
            chamber_stack.push(Chamber {
                offset: chamber.offset,
                h: divide + 1,
                w: chamber.w,
            });
            chamber_stack.push(Chamber {
                offset: maze::Point {
                    row: chamber.offset.row + divide,
                    col: chamber.offset.col,
                },
                h: chamber.h - divide,
                w: chamber.w,
            });
        } else if chamber.w > chamber.h && chamber.h > MIN_CHAMBER {
            let divide = random_even_div(&mut rng, chamber.w);
            let passage = rand_odd_pass(&mut rng, chamber.h);
            for r in 0..chamber.h {
                if r == passage {
                    continue;
                }
                build::build_wall_line_history(
                    &mut lk.maze,
                    maze::Point {
                        row: chamber.offset.row + r,
                        col: chamber.offset.col + divide,
                    },
                );
            }
            chamber_stack.push(Chamber {
                offset: chamber.offset,
                h: chamber.h,
                w: divide + 1,
            });
            chamber_stack.push(Chamber {
                offset: maze::Point {
                    row: chamber.offset.row,
                    col: chamber.offset.col + divide,
                },
                h: chamber.h,
                w: chamber.w - divide,
            });
        }
    }
}

///
/// Data only helpers.
///

fn random_even_div(rng: &mut ThreadRng, axis_limit: i32) -> i32 {
    2 * rng.gen_range(1..=((axis_limit - 2) / 2))
}

fn rand_odd_pass(rng: &mut ThreadRng, axis_limit: i32) -> i32 {
    2 * rng.gen_range(1..=((axis_limit - 2) / 2)) + 1
}
