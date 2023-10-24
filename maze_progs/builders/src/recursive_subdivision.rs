use crate::build;
use maze;
use speed;

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

// Public Functions-------------------------------------------------------------------------------

pub fn generate_maze(maze: &mut maze::Maze) {
    build::build_wall_outline(maze);
    let mut rng = thread_rng();
    let mut chamber_stack: Vec<Chamber> = Vec::from([Chamber {
        offset: maze::Point { row: 0, col: 0 },
        h: maze.row_size(),
        w: maze.col_size(),
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
                    maze,
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
                    maze,
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

pub fn animate_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    if maze.is_mini() {
        animate_mini_maze(maze, speed);
        return;
    }
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::build_wall_outline(maze);
    build::flush_grid(maze);
    build::print_overlap_key_animated(maze);
    let mut rng = thread_rng();
    let mut chamber_stack: Vec<Chamber> = Vec::from([Chamber {
        offset: maze::Point { row: 0, col: 0 },
        h: maze.row_size(),
        w: maze.col_size(),
    }]);
    while let Some(chamber) = chamber_stack.pop() {
        if maze.exit() {
            return;
        }
        if chamber.h >= chamber.w && chamber.w > MIN_CHAMBER {
            let divide = random_even_div(&mut rng, chamber.h);
            let passage = rand_odd_pass(&mut rng, chamber.w);
            for c in 0..chamber.w {
                if c == passage {
                    continue;
                }
                build::build_wall_line_animated(
                    maze,
                    maze::Point {
                        row: chamber.offset.row + divide,
                        col: chamber.offset.col + c,
                    },
                    animation,
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
                build::build_wall_line_animated(
                    maze,
                    maze::Point {
                        row: chamber.offset.row + r,
                        col: chamber.offset.col + divide,
                    },
                    animation,
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

fn animate_mini_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::build_wall_outline(maze);
    build::flush_grid(maze);
    build::print_overlap_key_animated(maze);
    let mut rng = thread_rng();
    let mut chamber_stack: Vec<Chamber> = Vec::from([Chamber {
        offset: maze::Point { row: 0, col: 0 },
        h: maze.row_size(),
        w: maze.col_size(),
    }]);
    while let Some(chamber) = chamber_stack.pop() {
        if maze.exit() {
            return;
        }
        if chamber.h >= chamber.w && chamber.w > MIN_CHAMBER {
            let divide = random_even_div(&mut rng, chamber.h);
            let passage = rand_odd_pass(&mut rng, chamber.w);
            for c in 0..chamber.w {
                if c == passage {
                    continue;
                }
                build::build_mini_wall_line_animated(
                    maze,
                    maze::Point {
                        row: chamber.offset.row + divide,
                        col: chamber.offset.col + c,
                    },
                    animation,
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
                build::build_mini_wall_line_animated(
                    maze,
                    maze::Point {
                        row: chamber.offset.row + r,
                        col: chamber.offset.col + divide,
                    },
                    animation,
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

// Private Helpers--------------------------------------------------------------------------------

fn random_even_div(rng: &mut ThreadRng, axis_limit: i32) -> i32 {
    2 * rng.gen_range(1..=((axis_limit - 2) / 2))
}

fn rand_odd_pass(rng: &mut ThreadRng, axis_limit: i32) -> i32 {
    2 * rng.gen_range(1..=((axis_limit - 2) / 2)) + 1
}
