mod builders;
mod solvers;
mod utilities;

pub use crate::utilities::maze;
pub use crate::utilities::print;
pub use crate::utilities::build;
pub use crate::utilities::solve;

pub use crate::builders::recursive_backtracker;
pub use crate::builders::kruskal;
pub use crate::builders::arena;

pub use crate::solvers::bfs;
pub use crate::solvers::dfs;
pub use crate::solvers::floodfs;
pub use crate::solvers::rdfs;

use std::collections::{HashMap, HashSet};
use std::env;

type BuildFunction = (
    fn(&mut maze::Maze),
    fn(&mut maze::Maze, build::BuilderSpeed),
);

type SolveFunction = (
    fn(maze::BoxMaze),
    fn(maze::BoxMaze, solve::SolverSpeed),
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
    build_speed: build::BuilderSpeed,
    build: BuildFunction,
    modify: Option<BuildFunction>,
    solve_view: ViewingMode,
    solve_speed: solve::SolverSpeed,
    solve: SolveFunction,
}

impl MazeRunner {
    fn default() -> Self {
        Self {
            args: maze::MazeArgs::default(),
            build_view: ViewingMode::StaticImage,
            build_speed: build::BuilderSpeed::Speed4,
            build: (
                recursive_backtracker::generate_maze,
                recursive_backtracker::animate_maze,
            ),
            modify: None,
            solve_view: ViewingMode::StaticImage,
            solve_speed: solve::SolverSpeed::Speed4,
            solve: (
                dfs::hunt,
                dfs::animate_hunt,
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
    build_animation_table: HashMap<String, build::BuilderSpeed>,
    solve_animation_table: HashMap<String, solve::SolverSpeed>,
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
        build_table: HashMap::from([
            (
                String::from("rdfs"),
                (
                    recursive_backtracker::generate_maze as fn(&mut maze::Maze),
                    recursive_backtracker::animate_maze as fn(&mut maze::Maze, build::BuilderSpeed),
                ),
            ),
            (
                String::from("kruskal"),
                (
                    kruskal::generate_maze as fn(&mut maze::Maze),
                    kruskal::animate_maze as fn(&mut maze::Maze, build::BuilderSpeed),
                ),
            ),
            (
                String::from("arena"),
                (
                    arena::generate_maze as fn(&mut maze::Maze),
                    arena::animate_maze as fn(&mut maze::Maze, build::BuilderSpeed),
                ),
            )
        ]),
        mod_table: HashMap::from([
            (
                String::from("cross"),
                (
                    build::add_cross as fn(&mut maze::Maze),
                    build::add_cross_animated as fn(&mut maze::Maze, build::BuilderSpeed),
                ),
            ),
            (
                String::from("x"),
                (
                    build::add_x as fn(&mut maze::Maze),
                    build::add_x_animated as fn(&mut maze::Maze, build::BuilderSpeed),
                ),
            ),
        ]),
        solve_table: HashMap::from([
            (
                String::from("dfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    dfs::animate_hunt as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("dfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    dfs::animate_gather as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("dfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    dfs::animate_corner as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("bfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    bfs::animate_hunt as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("bfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    bfs::animate_gather as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("bfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    bfs::animate_corner as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("floodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    floodfs::animate_hunt as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("floodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    floodfs::animate_gather as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("floodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    floodfs::animate_corner as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("rdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    rdfs::animate_hunt as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("rdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    rdfs::animate_gather as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
            (
                String::from("rdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    rdfs::animate_corner as fn(maze::BoxMaze, solve::SolverSpeed),
                ),
            ),
        ]),
        style_table: HashMap::from([
            (String::from("sharp"), maze::MazeStyle::Sharp),
            (String::from("round"), maze::MazeStyle::Round),
            (String::from("doubles"), maze::MazeStyle::Doubles),
            (String::from("bold"), maze::MazeStyle::Bold),
            (String::from("contrast"), maze::MazeStyle::Contrast),
            (String::from("spikes"), maze::MazeStyle::Spikes),
        ]),
        build_animation_table: HashMap::from([
            (String::from("0"), build::BuilderSpeed::Instant),
            (String::from("1"), build::BuilderSpeed::Speed1),
            (String::from("2"), build::BuilderSpeed::Speed2),
            (String::from("3"), build::BuilderSpeed::Speed3),
            (String::from("4"), build::BuilderSpeed::Speed4),
            (String::from("5"), build::BuilderSpeed::Speed5),
            (String::from("6"), build::BuilderSpeed::Speed6),
            (String::from("7"), build::BuilderSpeed::Speed7),
        ]),
        solve_animation_table: HashMap::from([
            (String::from("0"), solve::SolverSpeed::Instant),
            (String::from("1"), solve::SolverSpeed::Speed1),
            (String::from("2"), solve::SolverSpeed::Speed2),
            (String::from("3"), solve::SolverSpeed::Speed3),
            (String::from("4"), solve::SolverSpeed::Speed4),
            (String::from("5"), solve::SolverSpeed::Speed5),
            (String::from("6"), solve::SolverSpeed::Speed6),
            (String::from("7"), solve::SolverSpeed::Speed7),
        ]),
    };
    let mut run = MazeRunner::default();

    let args: Vec<String> = env::args().collect();
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for i in 1..args.len() {
        let a = &args[i];
        if process_current {
            set_args(
                &tables,
                &mut run,
                &FlagArg {
                    flag: prev_flag,
                    arg: &a,
                },
            );
            process_current = false;
            continue;
        }
        match tables.arg_flags.get(a) {
            Some(flag) => {
                process_current = true;
                prev_flag = flag;
            }
            None => {
                println!("Invalid argument flag: {}", a);
                print_usage();
                std::process::exit(1);
            }
        }
    }

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
    print::set_cursor_position(maze::Point { row: 0, col: 0 });

    match run.solve_view {
        ViewingMode::StaticImage => run.solve.0(maze),
        ViewingMode::AnimatedPlayback => run.solve.1(maze, run.solve_speed),
    }
}

fn set_args(tables: &LookupTables, run: &mut MazeRunner, pairs: &FlagArg) {
    match pairs.flag {
        "-h" => {
            print_usage();
            std::process::exit(0);
        }
        "-r" => set_rows(run, &pairs),
        "-c" => set_cols(run, &pairs),
        "-b" => match tables.build_table.get(pairs.arg) {
            Some(build_tuple) => run.build = *build_tuple,
            None => print_invalid_arg(pairs),
        },
        "-m" => match tables.mod_table.get(pairs.arg) {
            Some(mod_tuple) => run.modify = Some(*mod_tuple),
            None => print_invalid_arg(pairs),
        },
        "-s" => match tables.solve_table.get(pairs.arg) {
            Some(solve_tuple) => run.solve = *solve_tuple,
            None => print_invalid_arg(pairs),
        },
        "-d" => match tables.style_table.get(pairs.arg) {
            Some(wall_style) => run.args.style = *wall_style,
            None => print_invalid_arg(pairs),
        },
        "-ba" => match tables.build_animation_table.get(pairs.arg) {
            Some(speed) => {
                run.build_speed = *speed;
                run.build_view = ViewingMode::AnimatedPlayback;
            }
            None => {
                print_invalid_arg(pairs);
                std::process::exit(1);
            }
        },
        "-sa" => match tables.solve_animation_table.get(pairs.arg) {
            Some(speed) => {
                run.solve_speed = *speed;
                run.solve_view = ViewingMode::AnimatedPlayback;
            }
            None => print_invalid_arg(pairs),
        },
        _ => {
            print_invalid_arg(pairs);
            std::process::exit(1);
        }
    }
}

fn set_rows(run: &mut MazeRunner, pairs: &FlagArg) {
    let rows_result = pairs.arg.parse::<i32>();
    run.args.odd_rows = match rows_result {
        Ok(num) => {
            if num < 7 {
                print_invalid_arg(&pairs);
                std::process::exit(1);
            }
            num
        }
        Err(_) => {
            print_invalid_arg(&pairs);
            std::process::exit(1);
        }
    };
}

fn set_cols(run: &mut MazeRunner, pairs: &FlagArg) {
    let cols_result = pairs.arg.parse::<i32>();
    run.args.odd_cols = match cols_result {
        Ok(num) => {
            if num < 7 {
                print_invalid_arg(&pairs);
                std::process::exit(1);
            }
            num
        }
        Err(_) => {
            print_invalid_arg(&pairs);
            std::process::exit(1);
        }
    };
}

fn print_invalid_arg(pairs: &FlagArg) {
    println!("Flag was: {}", pairs.flag);
    println!("Argument was: {}", pairs.arg);
    print_usage();
    std::process::exit(1);
}

fn print_usage() {
    println!(
        "
    ┌───┬─────────┬─────┬───┬───────────┬─────┬───────┬─────────────┬─────┐
    │   │         │     │   │           │     │       │             │     │
    │ ╷ ╵ ┌───┐ ╷ └─╴ ╷ │ ╷ │ ┌─╴ ┌─┬─╴ │ ╶─┐ ╵ ┌───╴ │ ╶───┬─┐ ╶─┬─┘ ╶─┐ │
    │ │   │   │ │     │ │ │ │ │   │ │   │   │   │     │     │ │   │     │ │
    │ └───┤ ┌─┘ ├─────┘ └─┤ ╵ │ ┌─┘ ╵ ┌─┴─┐ └───┤ ╶───┴───┐ │ └─┐ ╵ ┌─┬─┘ │
    │     │ │   │      Thread Maze Usage Instructions     │ │   │   │ │   │
    ├───┐ ╵ │ -Use flags, followed by arguments, in any order:╷ └─┬─┘ │ ╷ │
    │   │   │ -r Rows flag. Set rows for the maze.    │   │ │ │   │   │ │ │
    │ ╶─┴─┐ └─┐ Any number > 7. Zoom out for larger mazes!╵ ╵ ├─┐ │ ╶─┤ └─┤
    │     │   -c Columns flag. Set columns for the maze.│     │ │ │   │   │
    │ ┌─┐ └─┐ │ Any number > 7. Zoom out for larger mazes!────┤ │ │ ╷ └─┐ │
    │ │ │   │ -b Builder flag. Set maze building algorithm.   │ │ │ │   │ │
    │ │ └─┐ ╵ │ rdfs - Randomized Depth First Search.         │ │ └─┘ ┌─┘ │
    │     │   │ kruskal - Randomized Kruskal's algorithm. │   │       │   │
    ├─────┤ ╷ ╵ prim - Randomized Prim's algorithm.─┴───┐ │ ┌─┴─────┬─┴─┐ │
    │     │ │   eller - Randomized Eller's algorithm.   │ │ │       │   │ │
    │     │ │   wilson - Loop-Erased Random Path Carver.│ │ │       │   │ │
    │ ┌─┐ ╵ ├─┬─wilson-walls - Loop-Erased Random Wall Adder. ┌───┐ ╵ ╷ │ │
    │ │ │   │ │ fractal - Randomized recursive subdivision. │ │   │   │ │ │
    │ ╵ ├───┘ ╵ grid - A random grid pattern. ├─┐ │ ┌─────┤ ╵ │ ┌─┴───┤ ╵ │
    │   │       arena - Open floor with no walls. │ │     │   │ │     │   │
    ├─╴ ├─────-m Modification flag. Add shortcuts to the maze.┘ │ ┌─┐ └─╴ │
    │   │     │ cross - Add crossroads through the center.      │ │ │     │
    │ ┌─┘ ┌─┐ │ x - Add an x of crossing paths through center.──┘ │ └─────┤
    │ │   │ │ -s Solver flag. Choose the game and solver. │ │     │       │
    │ ╵ ┌─┘ │ └─dfs-hunt - Depth First Search ╴ ┌───┴─┬─┘ │ │ ┌───┴─────┐ │
    │   │   │   dfs-gather - Depth First Search │     │   │ │ │         │ │
    ├───┘ ╶─┴─╴ dfs-corners - Depth First Search  ┌─╴ │ ╶─┼─┘ │ ╷ ┌───╴ ╵ │
    │           floodfs-hunt - Depth First Search │   │   │   │ │ │       │
    │ ┌───────┬─floodfs-gather - Depth First Search ┌─┴─╴ │ ╶─┴─┤ └───────┤
    │ │       │ floodfs-corners - Depth First Search│     │     │         │
    │ │ ╷ ┌─╴ │ rdfs-hunt - Randomized Depth First Search─┴─┬─╴ │ ┌─────╴ │
    │ │ │ │   │ rdfs-gather - Randomized Depth First Search │   │ │       │
    │ └─┤ └───┤ rdfs-corners - Randomized Depth First Search┤ ┌─┘ │ ╶───┐ │
    │   │     │ bfs-hunt - Breadth First Search     │   │   │ │   │     │ │
    ├─┐ │ ┌─┐ └─bfs-gather - Breadth First Search─┐ ╵ ╷ ├─╴ │ └─┐ ├───╴ │ │
    │ │ │ │ │   bfs-corners - Breadth First Search│   │ │   │   │ │     │ │
    │ │ │ ╵ └─-d Draw flag. Set the line style for the maze.┴─┐ └─┘ ┌─┬─┘ │
    │ │ │       sharp - The default straight lines. │   │     │     │ │   │
    │ │ └─┬───╴ round - Rounded corners.──╴ │ ╷ ╵ ╵ │ ╶─┴─┐ ╶─┴─────┘ │ ╶─┤
    │ │   │     doubles - Sharp double lines. │     │     │           │   │
    │ └─┐ └───┬─bold - Thicker straight lines.└─┬───┴─┬─╴ │ ┌───┬───╴ └─┐ │
    │   │     │ contrast - Full block width and height walls.   │       │ │
    │ ╷ ├─┬─╴ │ spikes - Connected lines with spikes. ╵ ┌─┘ ╵ ┌─┘ ┌─┐ ┌─┘ │
    │ │ │ │   -sa Solver Animation flag. Watch the maze solution. │ │ │   │
    │ │ ╵ │ ╶─┤ Any number 1-7. Speed increases with number.┌─┘ ┌─┤ ╵ │ ╶─┤
    │ │   │   -ba Builder Animation flag. Watch the maze build. │ │   │   │
    │ ├─╴ ├─┐ └─Any number 1-7. Speed increases with number.┘ ┌─┘ │ ┌─┴─┐ │
    │ │   │ │ -h Help flag. Make this prompt appear.  │   │   │   │ │   │ │
    │ └─┐ ╵ └─┐ No arguments.─┘ ┌───┐ └─┐ ├─╴ │ ╵ └───┤ ┌─┘ ┌─┴─╴ │ ├─╴ │ │
    │   │     -If any flags are omitted, defaults are used. │     │ │   │ │
    ├─╴ ├───┐ -Examples:┐ ╶─┬─┬─┘ ╷ ├─╴ │ │ ┌─┴───────┘ ├─╴ │ ╶─┐ │ ╵ ┌─┘ │
    │   │   │ │ cargo run   │ │   │ │   │ │ │           │   │   │ │   │   │
    │ ╶─┤ ╶─┘ │ cargo run -- -r 51 -c 111 -b rdfs -s bfs-hunt   │ │ ┌─┘ ┌─┤
    │   │     │ cargo run -- -c 111 -s bfs-gather       │   │   │ │ │   │ │
    │ ╷ │ ╶───┤ cargo run -- -s bfs-corners -d round -b fractal └─├─┤ ┌─┤ │
    │ │ │     │ cargo run -- -s dfs-hunt -ba 4 -b kruskal -m x    │ │ │ │ │
    ├─┘ ├───┬─┘ │ ╶─┼─╴ │ │ │ ╷ ├─┐ ╵ ╷ ├─┴───╴ │ │ ┌───┤   │ └─┐ ╵ └─┐ ╵ │
    │   │   │   │   │   │ │ │ │ │ │   │ │       │ │ │   │   │   │     │   │
    │ ╶─┘ ╷ ╵ ╶─┴───┘ ┌─┘ ╵ ╵ │ ╵ └───┤ ╵ ╶─────┘ │ ╵ ╷ └───┴─┐ └─────┴─╴ │
    │     │           │       │       │           │   │       │           │
    └─────┴───────────┴───────┴───────┴───────────┴───┴───────┴───────────┘
    "
    );
}
