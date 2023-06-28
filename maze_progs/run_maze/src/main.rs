use maze;
use print;

use builders::recursive_backtracker;
use builders::recursive_subdivision;
use builders::kruskal;
use builders::prim;
use builders::eller;
use builders::wilson_adder;
use builders::wilson_carver;
use builders::grid;
use builders::arena;
use builders::modify;

use solvers::dfs;
use solvers::rdfs;
use solvers::floodfs;
use solvers::bfs;

use std::collections::{HashMap, HashSet};
use std::env;

use ctrlc;

type BuildFunction = (
    fn(&mut maze::Maze),
    fn(&mut maze::Maze, speed::Speed),
);

type SolveFunction = (fn(maze::BoxMaze), fn(maze::BoxMaze, speed::Speed));

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
    build_speed: speed::Speed,
    build: BuildFunction,
    modify: Option<BuildFunction>,
    solve_view: ViewingMode,
    solve_speed: speed::Speed,
    solve: SolveFunction,
}

impl MazeRunner {
    fn default() -> Self {
        Self {
            args: maze::MazeArgs::default(),
            build_view: ViewingMode::StaticImage,
            build_speed: speed::Speed::Speed4,
            build: (
                recursive_backtracker::generate_maze,
                recursive_backtracker::animate_maze,
            ),
            modify: None,
            solve_view: ViewingMode::StaticImage,
            solve_speed: speed::Speed::Speed4,
            solve: (dfs::hunt, dfs::animate_hunt),
        }
    }
}

struct LookupTables {
    arg_flags: HashSet<String>,
    build_table: HashMap<String, BuildFunction>,
    mod_table: HashMap<String, BuildFunction>,
    solve_table: HashMap<String, SolveFunction>,
    style_table: HashMap<String, maze::MazeStyle>,
    animation_table: HashMap<String, speed::Speed>,
}

fn main() {
    // RAII approach to cursor hiding. Call hide and on scope drop it unhides, no call needed.
    let invisible = print::InvisibleCursor::new();
    invisible.hide();
    ctrlc::set_handler(move || {
        print::unhide_cursor_on_process_exit();
        std::process::exit(0);
    }).expect("Could not set quit handler.");

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
                    recursive_backtracker::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("fractal"),
                (
                    recursive_subdivision::generate_maze as fn(&mut maze::Maze),
                    recursive_subdivision::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("grid"),
                (
                    grid::generate_maze as fn(&mut maze::Maze),
                    grid::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("prim"),
                (
                    prim::generate_maze as fn(&mut maze::Maze),
                    prim::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("kruskal"),
                (
                    kruskal::generate_maze as fn(&mut maze::Maze),
                    kruskal::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("eller"),
                (
                    eller::generate_maze as fn(&mut maze::Maze),
                    eller::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("wilson"),
                (
                    wilson_carver::generate_maze as fn(&mut maze::Maze),
                    wilson_carver::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("wilson-walls"),
                (
                    wilson_adder::generate_maze as fn(&mut maze::Maze),
                    wilson_adder::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("arena"),
                (
                    arena::generate_maze as fn(&mut maze::Maze),
                    arena::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
        ]),
        mod_table: HashMap::from([
            (
                String::from("cross"),
                (
                    modify::add_cross as fn(&mut maze::Maze),
                    modify::add_cross_animated as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("x"),
                (
                    modify::add_x as fn(&mut maze::Maze),
                    modify::add_x_animated as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
        ]),
        solve_table: HashMap::from([
            (
                String::from("dfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    dfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("dfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    dfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("dfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    dfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    bfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    bfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    bfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    floodfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    floodfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    floodfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    rdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    rdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    rdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
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
        animation_table: HashMap::from([
            (String::from("0"), speed::Speed::Instant),
            (String::from("1"), speed::Speed::Speed1),
            (String::from("2"), speed::Speed::Speed2),
            (String::from("3"), speed::Speed::Speed3),
            (String::from("4"), speed::Speed::Speed4),
            (String::from("5"), speed::Speed::Speed5),
            (String::from("6"), speed::Speed::Speed6),
            (String::from("7"), speed::Speed::Speed7),
        ]),
    };
    let mut run = MazeRunner::default();

    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in env::args().skip(1) {
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
        match tables.arg_flags.get(&a) {
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
    if process_current {
        quit(&FlagArg { flag: &prev_flag, arg: "[NONE]" });
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
            None => quit(pairs),
        },
        "-m" => match tables.mod_table.get(pairs.arg) {
            Some(mod_tuple) => run.modify = Some(*mod_tuple),
            None => quit(pairs),
        },
        "-s" => match tables.solve_table.get(pairs.arg) {
            Some(solve_tuple) => run.solve = *solve_tuple,
            None => quit(pairs),
        },
        "-d" => match tables.style_table.get(pairs.arg) {
            Some(wall_style) => run.args.style = *wall_style,
            None => quit(pairs),
        },
        "-ba" => match tables.animation_table.get(pairs.arg) {
            Some(speed) => {
                run.build_speed = *speed;
                run.build_view = ViewingMode::AnimatedPlayback;
            }
            None => quit(pairs),
        },
        "-sa" => match tables.animation_table.get(pairs.arg) {
            Some(speed) => {
                run.solve_speed = *speed;
                run.solve_view = ViewingMode::AnimatedPlayback;
            }
            None => quit(pairs),
        },
        _ => quit(pairs),
    }
}

fn set_rows(run: &mut MazeRunner, pairs: &FlagArg) {
    run.args.odd_rows = match pairs.arg.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                quit(&pairs);
                std::process::exit(1);
            }
            num
        }
        Err(_) => {
            quit(&pairs);
            std::process::exit(1);
        }
    };
}

fn set_cols(run: &mut MazeRunner, pairs: &FlagArg) {
    run.args.odd_cols = match pairs.arg.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                quit(&pairs);
                std::process::exit(1);
            }
            num
        }
        Err(_) => {
            quit(&pairs);
            std::process::exit(1);
        }
    };
}

fn quit(pairs: &FlagArg) {
    println!("Flag was: {}", pairs.flag);
    println!("Argument was: {}", pairs.arg);
    print_usage();
    print::maze_panic!("");
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
    ├─╴ ├───┐ -Flags Following [cargo run]: ┌─┴───────┘ ├─╴ │ ╶─┐ │ ╵ ┌─┘ │
    │   │   │ │ --release --bin run_maze│ │ │           │   │   │ │   │   │
    │ ╶─┤ ╶─┘ │ --release --bin run_maze -- -r 51 -c 111 -s bfs-hunt┌─┘ ┌─┤
    │   │     │ --release --bin run_maze -- -c 111 -s bfs-gather│ │ │   │ │
    │ ╷ │ ╶───┤ --release --bin run_maze -- -b prim -sa 1       └─┼─┤ ┌─┤ │
    │ │ │     │ --release --bin run_maze -- -ba 4 -b kruskal -m x │ │ │ │ │
    ├─┘ ├───┬─┘ │ ╶─┼─╴ │ │ │ ╷ ├─┐ ╵ ╷ ├─┴───╴ │ │ ┌───┤   │ └─┐ ╵ └─┐ ╵ │
    │   │   │   │   │   │ │ │ │ │ │   │ │       │ │ │   │   │   │     │   │
    │ ╶─┘ ╷ ╵ ╶─┴───┘ ┌─┘ ╵ ╵ │ ╵ └───┤ ╵ ╶─────┘ │ ╵ ╷ └───┴─┐ └─────┴─╴ │
    │     │           │       │       │           │   │       │           │
    └─────┴───────────┴───────┴───────┴───────────┴───┴───────┴───────────┘
    "
    );
}
