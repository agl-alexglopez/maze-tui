mod builders;
mod solvers;
mod utilities;

pub use crate::builders::recursive_backtracker;
pub use crate::solvers::dfs;
pub use crate::utilities::build_util;
pub use crate::utilities::maze;
pub use crate::utilities::print_util;
pub use crate::utilities::solve_util;

use std::collections::{HashMap, HashSet};

type BuildFunction = (
    fn(&mut maze::Maze),
    fn(&mut maze::Maze, build_util::BuilderSpeed),
);

type SolveFunction = (
    fn(maze::BoxMaze),
    fn(maze::BoxMaze, solve_util::SolverSpeed),
);

struct FlagArg<'a, 'b> {
    flag: &'a str,
    arg: &'b str,
}

enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

struct MazeRunner {
    args: maze::MazeArgs,
    build_view: ViewingMode,
    build_speed: build_util::BuilderSpeed,
    build: BuildFunction,
    modify: Option<BuildFunction>,
    solve_view: ViewingMode,
    solve_speed: solve_util::SolverSpeed,
    solve: SolveFunction,
}

impl MazeRunner {
    fn default() -> Self {
        Self {
            args: maze::MazeArgs::default(),
            build_view: ViewingMode::StaticImage,
            build_speed: build_util::BuilderSpeed::Speed4,
            build: (
                recursive_backtracker::generate_recursive_backtracker_maze,
                recursive_backtracker::animate_recursive_backtracker_maze,
            ),
            modify: None,
            solve_view: ViewingMode::StaticImage,
            solve_speed: solve_util::SolverSpeed::Speed4,
            solve: (
                dfs::solve_with_dfs_thread_hunt,
                dfs::animate_with_dfs_thread_hunt,
            ),
        }
    }
}

struct LookupTables {
    arg_flags: HashSet<String>,
    build_table: HashMap<String, BuildFunction>,
    mod_table: HashMap<String, BuildFunction>,
    solve_table: HashMap<String, SolveFunction>,
    style_table: HashMap<String, maze::MazeStyle>,
    build_animation_table: HashMap<String, build_util::BuilderSpeed>,
    solve_animation_table: HashMap<String, solve_util::SolverSpeed>,
}

fn main() {
    let tables = LookupTables {
        arg_flags: HashSet::from([
            String::from("-r"),
            String::from("-c"),
            String::from("-b"),
            String::from("-s"),
            String::from("-h"),
            String::from("-g"),
            String::from("-d"),
            String::from("-m"),
            String::from("-sa"),
            String::from("-ba"),
        ]),
        build_table: HashMap::from([(
            String::from("rdfs"),
            (
                recursive_backtracker::generate_recursive_backtracker_maze as fn(&mut maze::Maze),
                recursive_backtracker::animate_recursive_backtracker_maze
                    as fn(&mut maze::Maze, build_util::BuilderSpeed),
            ),
        )]),
        mod_table: HashMap::from([
            (
                String::from("cross"),
                (
                    build_util::add_cross as fn(&mut maze::Maze),
                    build_util::add_cross_animated as fn(&mut maze::Maze, build_util::BuilderSpeed),
                ),
            ),
            (
                String::from("x"),
                (
                    build_util::add_x as fn(&mut maze::Maze),
                    build_util::add_x_animated as fn(&mut maze::Maze, build_util::BuilderSpeed),
                ),
            ),
        ]),
        solve_table: HashMap::from([(
            String::from("dfs-hunt"),
            (
                dfs::solve_with_dfs_thread_hunt as fn(maze::BoxMaze),
                dfs::animate_with_dfs_thread_hunt as fn(maze::BoxMaze, solve_util::SolverSpeed),
            ),
        )]),
        style_table: HashMap::from([
            (String::from("sharp"), maze::MazeStyle::Sharp),
            (String::from("round"), maze::MazeStyle::Round),
            (String::from("doubles"), maze::MazeStyle::Doubles),
            (String::from("bold"), maze::MazeStyle::Bold),
            (String::from("contrast"), maze::MazeStyle::Contrast),
            (String::from("spikes"), maze::MazeStyle::Spikes),
        ]),
        build_animation_table: HashMap::from([
            (String::from("0"), build_util::BuilderSpeed::Instant),
            (String::from("1"), build_util::BuilderSpeed::Speed1),
            (String::from("2"), build_util::BuilderSpeed::Speed2),
            (String::from("3"), build_util::BuilderSpeed::Speed3),
            (String::from("4"), build_util::BuilderSpeed::Speed4),
            (String::from("5"), build_util::BuilderSpeed::Speed5),
            (String::from("6"), build_util::BuilderSpeed::Speed6),
            (String::from("7"), build_util::BuilderSpeed::Speed7),
        ]),
        solve_animation_table: HashMap::from([
            (String::from("0"), solve_util::SolverSpeed::Instant),
            (String::from("1"), solve_util::SolverSpeed::Speed1),
            (String::from("2"), solve_util::SolverSpeed::Speed2),
            (String::from("3"), solve_util::SolverSpeed::Speed3),
            (String::from("4"), solve_util::SolverSpeed::Speed4),
            (String::from("5"), solve_util::SolverSpeed::Speed5),
            (String::from("6"), solve_util::SolverSpeed::Speed6),
            (String::from("7"), solve_util::SolverSpeed::Speed7),
        ]),
    };
    let run = MazeRunner::default();
    let mut maze = maze::Maze::new(run.args);

    match run.build_view {
        ViewingMode::StaticImage => {
            run.build.0(&mut maze);
            match run.modify {
                Some((static_mod, _)) => static_mod(&mut maze),
                None => {}
            }
        }
        ViewingMode::AnimatedPlayback => {
            run.build.1(&mut maze, run.build_speed);
            match run.modify {
                Some((_, animate_mod)) => animate_mod(&mut maze, run.build_speed),
                None => {}
            }
        }
    }

    // Ensure a smooth transition from build to solve with no flashing.
    print_util::set_cursor_position(maze::Point { row: 0, col: 0 });

    match run.solve_view {
        ViewingMode::StaticImage => run.solve.0(maze),
        ViewingMode::AnimatedPlayback => run.solve.1(maze, run.solve_speed),
    }
}
