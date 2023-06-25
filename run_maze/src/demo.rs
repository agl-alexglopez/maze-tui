mod builders;
mod solvers;
mod utilities;

pub use crate::utilities::build;
pub use crate::utilities::maze;
pub use crate::utilities::print;
pub use crate::utilities::solve;

pub use crate::builders::arena;
pub use crate::builders::eller;
pub use crate::builders::grid;
pub use crate::builders::kruskal;
pub use crate::builders::prim;
pub use crate::builders::recursive_backtracker;
pub use crate::builders::recursive_subdivision;
pub use crate::builders::wilson_adder;
pub use crate::builders::wilson_carver;

pub use crate::solvers::bfs;
pub use crate::solvers::dfs;
pub use crate::solvers::floodfs;
pub use crate::solvers::rdfs;

use std::{thread, time};
use std::io::{stdout, Write};
use std::env;

use rand::{thread_rng, distributions::{Bernoulli, Distribution}, seq::SliceRandom};

type BuildDemo = fn(&mut maze::Maze, build::BuilderSpeed);

type SolveDemo = fn(maze::BoxMaze, solve::SolverSpeed);

struct DemoRunner {
    args: maze::MazeArgs,
    builder_speed: Vec<build::BuilderSpeed>,
    wall_styles: Vec<maze::MazeStyle>,
    builders: Vec<BuildDemo>,
    modifications: Vec<BuildDemo>,
    solver_speed: Vec<solve::SolverSpeed>,
    solvers: Vec<SolveDemo>,
}

impl DemoRunner {
    fn default() -> Self {
        Self {
            args: maze::MazeArgs::default(),
            builder_speed: vec![
                build::BuilderSpeed::Speed1,
                build::BuilderSpeed::Speed2,
                build::BuilderSpeed::Speed3,
                build::BuilderSpeed::Speed4,
                build::BuilderSpeed::Speed5,
                build::BuilderSpeed::Speed6,
                build::BuilderSpeed::Speed7,
            ],
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
            modifications: vec![build::add_cross_animated, build::add_x_animated],
            solver_speed: vec![
                solve::SolverSpeed::Speed1,
                solve::SolverSpeed::Speed2,
                solve::SolverSpeed::Speed3,
                solve::SolverSpeed::Speed4,
                solve::SolverSpeed::Speed5,
                solve::SolverSpeed::Speed6,
                solve::SolverSpeed::Speed7,
            ],
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
                _ => panic!("Bad flag snuck past first check?")
            }
            process_current = false;
            continue;
        }
        match a.as_str() {
            "-r" => prev_flag = "-r",
            "-c" => prev_flag = "-c",
            _ => panic!("May only specify row or col size for this program (-r or -c)"),
        }
        process_current = true;
    }
    let mut rng = thread_rng();
    let modification_probability = Bernoulli::new(0.2);
    loop {
        match run.wall_styles.choose(&mut rng) {
            Some(&style) => run.args.style = style,
            None => panic!("Styles not set for loop, broken"),
        }
        let mut maze = maze::Maze::new(run.args);
        let build_speed = match run.builder_speed.choose(&mut rng) {
            Some(&speed) => speed,
            None => panic!("Build speed array empty."),
        };
        let solve_speed = match run.solver_speed.choose(&mut rng) {
            Some(&speed) => speed,
            None => panic!("Solve speed array empty."),
        };
        let build_algo = match run.builders.choose(&mut rng) {
            Some(&algo) => algo,
            None => panic!("Build algo array empty."),
        };
        let solve_algo = match run.solvers.choose(&mut rng) {
            Some(&algo) => algo,
            None => panic!("Build algo array empty."),
        };

        build_algo(&mut maze, build_speed);

        if modification_probability.expect("Bernoulli innefective").sample(&mut rng) {
           match run.modifications.choose(&mut rng) {
               Some(modder) => {
                   modder(&mut maze, build_speed);
               }
               None => panic!("Empty modification table.")
           }
        }

        print::set_cursor_position(maze::Point { row: 0, col: 0 });

        solve_algo(maze, solve_speed);
        print!("Loading next maze...");
        stdout().flush().expect("Couldn't flush stdout");
        thread::sleep(time::Duration::from_secs(2));
        print::set_cursor_position(maze::Point { row: 0, col: 0 });
    }
}

fn set_rows(run: &mut DemoRunner, size: &str) {
    match size.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                panic!("Demo can only run larger than 7x7");
            }
            run.args.odd_rows = num + 1 - (num % 2);
        }
        Err(_) => {
            panic!("Invalid row size: {}", size);
        }
    }
}

fn set_cols(run: &mut DemoRunner, size: &str) {
    match size.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                panic!("Demo can only run larger than 7x7");
            }
            run.args.odd_cols = num + 1 - (num % 2);
        }
        Err(_) => {
            panic!("Invalid col size: {}", size);
        }
    }
}
