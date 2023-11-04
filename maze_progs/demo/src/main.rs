use builders::build;

use crossbeam_channel::bounded;
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
    let (impatient_user, worker) = bounded::<bool>(1);
    invisible.hide();
    let impatient_clone = impatient_user.clone();
    ctrlc::set_handler(move || {
        print::unhide_cursor_on_process_exit();
        if impatient_clone.send(true).is_err() {
            std::process::exit(0);
        }
    })
    .expect("Could not set quit handler.");
    loop {
        let mut run = tables::CursorRunner::new();
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
        let maze = maze::Maze::new(run.args);
        let build_speed = match tables::SPEEDS.choose(&mut rng) {
            Some(&(_key, val)) => val,
            None => print::maze_panic!("Build speed array empty."),
        };
        let solve_speed = match tables::SPEEDS.choose(&mut rng) {
            Some(&(_key, val)) => val,
            None => print::maze_panic!("Solve speed array empty."),
        };
        let build_algo = match tables::CURSOR_BUILDERS.choose(&mut rng) {
            Some(&(_key, val)) => val.1,
            None => print::maze_panic!("Build algo array empty."),
        };
        let solve_algo = match tables::CURSOR_SOLVERS.choose(&mut rng) {
            Some(&(_key, val)) => val.1,
            None => print::maze_panic!("Build algo array empty."),
        };
        build::print_overlap_key(&maze);
        let monitor = monitor::MazeReceiver::new(maze, worker.clone());
        build_algo(monitor.clone(), build_speed);

        if modification_probability
            .expect("Bernoulli innefective")
            .sample(&mut rng)
        {
            match tables::CURSOR_MODIFICATIONS.choose(&mut rng) {
                Some((_key, val)) => {
                    val.1(monitor.clone(), build_speed);
                }
                None => print::maze_panic!("Empty modification table."),
            }
        }

        print::set_cursor_position(maze::Point::default(), maze::Offset::default());

        solve_algo(monitor.clone(), solve_speed);
        if let Ok(lk) = monitor.clone().solver.lock() {
            print::set_cursor_position(
                maze::Point {
                    row: if lk.maze.style_index() == (maze::MazeStyle::Mini as usize) {
                        lk.maze.row_size() / 2 + 3
                    } else {
                        lk.maze.row_size() + 2
                    },
                    col: 0,
                },
                maze::Offset::default(),
            );
        }
        if impatient_user.is_full() {
            break;
        }
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
