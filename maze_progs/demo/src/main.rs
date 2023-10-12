use builders::arena;
use builders::eller;
use builders::grid;
use builders::kruskal;
use builders::modify;
use builders::prim;
use builders::recursive_backtracker;
use builders::recursive_subdivision;
use builders::wilson_adder;
use builders::wilson_carver;

use solvers::bfs;
use solvers::dfs;
use solvers::floodfs;
use solvers::rdfs;

use painters::distance;
use painters::runs;

use std::env;
use std::{thread, time};

use rand::{
    distributions::{Bernoulli, Distribution},
    seq::SliceRandom,
    thread_rng,
};

type BuildDemo = fn(&mut maze::Maze, speed::Speed);

type SolveDemo = fn(maze::BoxMaze, speed::Speed);

struct DemoRunner {
    args: maze::MazeArgs,
    wall_styles: Vec<maze::MazeStyle>,
    builders: Vec<BuildDemo>,
    modifications: Vec<BuildDemo>,
    solvers: Vec<SolveDemo>,
    speeds: Vec<speed::Speed>,
}

impl DemoRunner {
    fn default() -> Self {
        Self {
            args: maze::MazeArgs::default(),
            wall_styles: vec![
                maze::MazeStyle::Sharp,
                maze::MazeStyle::Round,
                maze::MazeStyle::Doubles,
                maze::MazeStyle::Bold,
                maze::MazeStyle::Contrast,
                maze::MazeStyle::Spikes,
            ],
            builders: vec![
                arena::animate_maze,
                recursive_backtracker::animate_maze,
                recursive_subdivision::animate_maze,
                prim::animate_maze,
                kruskal::animate_maze,
                eller::animate_maze,
                wilson_carver::animate_maze,
                wilson_adder::animate_maze,
                grid::animate_maze,
            ],
            modifications: vec![modify::add_cross_animated, modify::add_x_animated],
            solvers: vec![
                dfs::animate_hunt,
                dfs::animate_gather,
                dfs::animate_corner,
                rdfs::animate_hunt,
                rdfs::animate_gather,
                rdfs::animate_corner,
                bfs::animate_hunt,
                bfs::animate_gather,
                bfs::animate_corner,
                floodfs::animate_hunt,
                floodfs::animate_gather,
                floodfs::animate_corner,
                runs::animate_run_lengths,
                distance::animate_distance_from_center,
            ],
            speeds: vec![
                speed::Speed::Speed1,
                speed::Speed::Speed2,
                speed::Speed::Speed3,
                speed::Speed::Speed4,
                speed::Speed::Speed5,
                speed::Speed::Speed6,
                speed::Speed::Speed7,
            ],
        }
    }
}

fn main() {
    let mut run = DemoRunner::default();
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in env::args().skip(1) {
        if process_current {
            match prev_flag {
                "-r" => set_rows(&mut run, &a),
                "-c" => set_cols(&mut run, &a),
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
        match run.wall_styles.choose(&mut rng) {
            Some(&style) => run.args.style = style,
            None => print::maze_panic!("Styles not set for loop, broken"),
        }
        let mut maze = maze::Maze::new(run.args);
        let build_speed = match run.speeds.choose(&mut rng) {
            Some(&speed) => speed,
            None => print::maze_panic!("Build speed array empty."),
        };
        let solve_speed = match run.speeds.choose(&mut rng) {
            Some(&speed) => speed,
            None => print::maze_panic!("Solve speed array empty."),
        };
        let build_algo = match run.builders.choose(&mut rng) {
            Some(&algo) => algo,
            None => print::maze_panic!("Build algo array empty."),
        };
        let solve_algo = match run.solvers.choose(&mut rng) {
            Some(&algo) => algo,
            None => print::maze_panic!("Build algo array empty."),
        };

        build_algo(&mut maze, build_speed);

        if modification_probability
            .expect("Bernoulli innefective")
            .sample(&mut rng)
        {
            match run.modifications.choose(&mut rng) {
                Some(modder) => {
                    modder(&mut maze, build_speed);
                }
                None => print::maze_panic!("Empty modification table."),
            }
        }

        print::set_cursor_position(maze::Point::default(), maze::Offset::default());

        solve_algo(maze, solve_speed);
        print!("Loading next maze...");
        print::flush();
        thread::sleep(time::Duration::from_secs(2));
        print::set_cursor_position(maze::Point::default(), maze::Offset::default());
    }
}

fn set_rows(run: &mut DemoRunner, size: &str) {
    match size.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                print::maze_panic!("Demo can only run larger than 7x7");
            }
            run.args.odd_rows = num + 1 - (num % 2);
        }
        Err(_) => {
            print::maze_panic!("Invalid row size: {}", size);
        }
    }
}

fn set_cols(run: &mut DemoRunner, size: &str) {
    match size.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                print::maze_panic!("Demo can only run larger than 7x7");
            }
            run.args.odd_cols = num + 1 - (num % 2);
        }
        Err(_) => {
            print::maze_panic!("Invalid col size: {}", size);
        }
    }
}
