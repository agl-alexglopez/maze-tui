use builders::build;
use maze;
use solvers::solve;
use tables;

use std::env;
use std::{thread, time};

use rand::{
    distributions::{Bernoulli, Distribution},
    seq::SliceRandom,
    thread_rng,
};

fn main() {
    let mut prev_flag: &str = "";
    let mut process_current = false;
    let mut dimensions = (33, 111);
    for a in env::args().skip(1) {
        if process_current {
            match prev_flag {
                "-r" => dimensions.0 = set_dimension(&a),
                "-c" => dimensions.1 = set_dimension(&a),
                _ => print::maze_panic!("Bad flag snuck past first check?"),
            }
            process_current = false;
            continue;
        }
        match a.as_str() {
            "-r" => prev_flag = "-r",
            "-c" => prev_flag = "-c",
            _ => print::maze_panic!("May only specify row or col size for this program (-r or -c)"),
        }
        process_current = true;
    }
    let mut rng = thread_rng();
    let modification_probability = Bernoulli::new(0.2);
    let invisible = print::InvisibleCursor::new();
    invisible.hide();
    ctrlc::set_handler(move || {
        print::clear_screen();
        print::set_cursor_position(
            maze::Point { row: 0, col: 0 },
            maze::Offset {
                add_rows: 0,
                add_cols: 0,
            },
        );
        print::unhide_cursor_on_process_exit();
        std::process::exit(0);
    })
    .expect("Could not set quit handler.");
    loop {
        let mut run = tables::MazeRunner::new();
        run.args.odd_rows = dimensions.0;
        run.args.odd_cols = dimensions.1;
        print::clear_screen();
        match tables::WALL_STYLES.choose(&mut rng) {
            Some(&(_key, val)) => run.args.style = val,
            None => print::maze_panic!("Styles not set for loop, broken"),
        }
        if run.args.style == maze::MazeStyle::Mini {
            run.args.odd_rows *= 2;
        }
        let mut maze = maze::Maze::new(run.args);
        let build_speed = match tables::SPEEDS.choose(&mut rng) {
            Some(&(_key, val)) => val,
            None => print::maze_panic!("Build speed array empty."),
        };
        let solve_speed = match tables::SPEEDS.choose(&mut rng) {
            Some(&(_key, val)) => val,
            None => print::maze_panic!("Solve speed array empty."),
        };
        let build_algo = match tables::BUILDERS.choose(&mut rng) {
            Some(&(_key, val)) => val.1,
            None => print::maze_panic!("Build algo array empty."),
        };
        let solve_algo = match tables::SOLVERS.choose(&mut rng) {
            Some(&(_key, val)) => val.1,
            None => print::maze_panic!("Build algo array empty."),
        };

        build::print_overlap_key_animated(&maze);
        build_algo(&mut maze, build_speed);

        if modification_probability
            .expect("Bernoulli innefective")
            .sample(&mut rng)
        {
            match tables::MODIFICATIONS.choose(&mut rng) {
                Some((_key, val)) => {
                    val.1(&mut maze, build_speed);
                }
                None => print::maze_panic!("Empty modification table."),
            }
        }

        print::set_cursor_position(maze::Point::default(), maze::Offset::default());

        let monitor = solve::Solver::new(maze);
        solve_algo(monitor, solve_speed);
        print::set_cursor_position(
            maze::Point {
                row: dimensions.0 + 3,
                col: 0,
            },
            maze::Offset::default(),
        );
        print!("Loading next maze...");
        print::flush();
        thread::sleep(time::Duration::from_secs(2));
        print::set_cursor_position(maze::Point::default(), maze::Offset::default());
    }
}

fn set_dimension(size: &str) -> i32 {
    match size.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                print::maze_panic!("Demo can only run larger than 7x7");
            }
            num + 1 - (num % 2)
        }
        Err(_) => {
            print::maze_panic!("Invalid row size: {}", size);
        }
    }
}
