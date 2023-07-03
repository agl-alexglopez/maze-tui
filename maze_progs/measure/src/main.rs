use maze;
use print;

use builders::arena;
use builders::build::clear_and_flush_grid;
use builders::eller;
use builders::grid;
use builders::kruskal;
use builders::modify;
use builders::prim;
use builders::recursive_backtracker;
use builders::recursive_subdivision;
use builders::wilson_adder;
use builders::wilson_carver;

use painters::distance;
use painters::runs;

use std::collections::{HashMap, HashSet};
use std::env;

use ctrlc;

type BuildFunction = (fn(&mut maze::Maze), fn(&mut maze::Maze, speed::Speed));
type PaintFunction = (fn(maze::BoxMaze), fn(maze::BoxMaze, speed::Speed));

struct FlagArg<'a, 'b> {
    flag: &'a str,
    arg: &'b str,
}

enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

struct MeasurementRunner {
    args: maze::MazeArgs,
    build_view: ViewingMode,
    build_speed: speed::Speed,
    build: BuildFunction,
    modify: Option<BuildFunction>,
    paint_view: ViewingMode,
    paint_speed: speed::Speed,
    paint: PaintFunction,
}

impl MeasurementRunner {
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
            paint_view: ViewingMode::StaticImage,
            paint_speed: speed::Speed::Speed4,
            paint: (
                distance::paint_distance_from_center,
                distance::animate_distance_from_center,
            ),
        }
    }
}

struct LookupTables {
    arg_flags: HashSet<String>,
    build_table: HashMap<String, BuildFunction>,
    mod_table: HashMap<String, BuildFunction>,
    paint_table: HashMap<String, PaintFunction>,
    style_table: HashMap<String, maze::MazeStyle>,
    animation_table: HashMap<String, speed::Speed>,
}

fn main() {
    let invisible = print::InvisibleCursor::new();
    invisible.hide();
    ctrlc::set_handler(move || {
        print::clear_screen();
        print::set_cursor_position(maze::Point { row: 0, col: 0 });
        print::unhide_cursor_on_process_exit();
        std::process::exit(0);
    })
    .expect("Could not set quit handler.");

    let tables = LookupTables {
        arg_flags: HashSet::from([
            String::from("-r"),
            String::from("-c"),
            String::from("-b"),
            String::from("-p"),
            String::from("-h"),
            String::from("-d"),
            String::from("-m"),
            String::from("-pa"),
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
        paint_table: HashMap::from([
            (
                String::from("distance"),
                (
                    distance::paint_distance_from_center as fn(maze::BoxMaze),
                    distance::animate_distance_from_center as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("runs"),
                (
                    runs::paint_run_lengths as fn(maze::BoxMaze),
                    runs::animate_run_lengths as fn(maze::BoxMaze, speed::Speed),
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
    let mut measure = MeasurementRunner::default();
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in env::args().skip(1) {
        if process_current {
            set_args(
                &tables,
                &mut measure,
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
                quit(&FlagArg {
                    flag: &a,
                    arg: "[NONE]",
                });
            }
        }
    }
    if process_current {
        quit(&FlagArg {
            flag: &prev_flag,
            arg: "[NONE]",
        });
    }

    let mut maze = maze::Maze::new(measure.args);

    match measure.build_view {
        ViewingMode::StaticImage => {
            measure.build.0(&mut maze);
            clear_and_flush_grid(&maze);
            match measure.modify {
                Some((static_mod, _)) => static_mod(&mut maze),
                None => {}
            }
        }
        ViewingMode::AnimatedPlayback => {
            measure.build.1(&mut maze, measure.build_speed);
            match measure.modify {
                Some((_, animate_mod)) => animate_mod(&mut maze, measure.build_speed),
                None => {}
            }
        }
    }

    // Ensure a smooth transition from build to solve with no flashing.
    print::set_cursor_position(maze::Point { row: 0, col: 0 });

    match measure.paint_view {
        ViewingMode::StaticImage => measure.paint.0(maze),
        ViewingMode::AnimatedPlayback => measure.paint.1(maze, measure.paint_speed),
    }
}

fn set_args(tables: &LookupTables, measure: &mut MeasurementRunner, pairs: &FlagArg) {
    match pairs.flag {
        "-h" => {
            print_usage();
            safe_exit();
        }
        "-r" => measure.args.odd_rows = set_dimension(&pairs),
        "-c" => measure.args.odd_cols = set_dimension(&pairs),
        "-b" => match tables.build_table.get(pairs.arg) {
            Some(build_tuple) => measure.build = *build_tuple,
            None => quit(pairs),
        },
        "-m" => match tables.mod_table.get(pairs.arg) {
            Some(mod_tuple) => measure.modify = Some(*mod_tuple),
            None => quit(pairs),
        },
        "-p" => match tables.paint_table.get(pairs.arg) {
            Some(solve_tuple) => measure.paint = *solve_tuple,
            None => quit(pairs),
        },
        "-d" => match tables.style_table.get(pairs.arg) {
            Some(wall_style) => measure.args.style = *wall_style,
            None => quit(pairs),
        },
        "-ba" => match tables.animation_table.get(pairs.arg) {
            Some(speed) => {
                measure.build_speed = *speed;
                measure.build_view = ViewingMode::AnimatedPlayback;
            }
            None => quit(pairs),
        },
        "-pa" => match tables.animation_table.get(pairs.arg) {
            Some(speed) => {
                measure.paint_speed = *speed;
                measure.paint_view = ViewingMode::AnimatedPlayback;
            }
            None => quit(pairs),
        },
        _ => quit(pairs),
    };
}

fn set_dimension(pairs: &FlagArg) -> i32 {
    match pairs.arg.parse::<i32>() {
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
    }
}

fn safe_exit() {
    print::unhide_cursor_on_process_exit();
    std::process::exit(0);
}

fn quit(pairs: &FlagArg) {
    println!("Flag was: {}", pairs.flag);
    println!("Argument was: {}", pairs.arg);
    print_usage();
    print::unhide_cursor_on_process_exit();
    std::process::exit(0);
}

fn print_usage() {
    println!(
        "
    ┌───┬─────────┬─────┬───┬───────────┬─────┬───────┬─────────────┬─────┐
    │   │         │     │   │           │     │       │             │     │
    │ ╷ ╵ ┌───┐ ╷ └─╴ ╷ │ ╷ │ ┌─╴ ┌─┬─╴ │ ╶─┐ ╵ ┌───╴ │ ╶───┬─┐ ╶─┬─┘ ╶─┐ │
    │ │   │   │ │     │ │ │ │ │   │ │   │   │   │     │     │ │   │     │ │
    │ └───┤ ┌─┘ ├─────┘ └─┤ ╵ │ ┌─┘ ╵ ┌─┴─┐ └───┤ ╶───┴───┐ │ └─┐ ╵ ┌─┬─┘ │
    │     │ │   │       Maze Measurement Instructions     │ │   │   │ │   │
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
    │ │   │ │ -p Paint flag. Choose the metric to measure and paint.      │
    │ │ ┌─┘ │ │ distance - Distance from the center─┴─┬─┘ │ │     │       │
    │ │ │   │   runs - Run lengths of straight passages.  │ │    ─┴─────┐ │
    │ │ │   └─-d Draw flag. Set the line style for the maze.┴─┐     ┌─┬─┘ │
    │ │ │       sharp - The default straight lines. │   │     │     │ │   │
    │ │ └─┬───╴ round - Rounded corners.──────┐     │ ╶─┴─┐ ╶─┴─────┘ │ ╶─┤
    │ │   │     doubles - Sharp double lines. │     │     │           │   │
    │ └─┐ └───┬─bold - Thicker straight lines.└─┬───┴─┬─╴ │ ┌───┬───╴ └─┐ │
    │   │     │ contrast - Full block width and height walls.   │       │ │
    │ ╷ ├─┬─╴ │ spikes - Connected lines with spikes. ╵ ┌─┘ ╵ ┌─┘ ┌─┐ ┌─┘ │
    │ │ │ │   -pa Paint Animation flag. Watch the measurement.│   │ │ │   │
    │ │ ╵ │ ╶─┤ Any number 1-7. Speed increases with number.┌─┘ ┌─┤ ╵ │ ╶─┤
    │ │   │   -ba Builder Animation flag. Watch the maze build. │ │   │   │
    │ ├─╴ ├─┐ └─Any number 1-7. Speed increases with number.┘ ┌─┘ │ ┌─┴─┐ │
    │ │   │ │ -h Help flag. Make this prompt appear.  │   │   │   │ │   │ │
    │ └─┐ ╵ └─┐ No arguments.─┘ ┌───┐ └─┐ ├─╴ │ ╵ └───┤ ┌─┘ ┌─┴─╴ │ ├─╴ │ │
    │   │     -If any flags are omitted, defaults are used. │     │ │   │ │
    ├─╴ ├───┐ -Flags Following [cargo run]: ┌─┴───────┘ ├─╴ │ ╶─┐ │ ╵ ┌─┘ │
    │   │   │ │ --release --bin measure │ │ │           │   │   │ │   │   │
    │ ╶─┤ ╶─┘ │ --release --bin measure -- -r 51 -c 111 -p measure│ ┌─┘ ┌─┤
    │   │     │ --release --bin measure -- -c 111 -s measure    │ │ │   │ │
    │ ╷ │ ╶───┤ --release --bin measure -- -b prim -pa 1        └─┼─┤ ┌─┤ │
    │ │ │     │ --release --bin measure -- -ba 4 -b kruskal -m x  │ │ │ │ │
    ├─┘ ├───┬─┘ │ ╶─┼─╴ │ │ │ ╷ ├─┐ ╵ ╷ ├─┴───╴ │ │ ┌───┤   │ └─┐ ╵ └─┐ ╵ │
    │   │   │   │   │   │ │ │ │ │ │   │ │       │ │ │   │   │   │     │   │
    │ ╶─┘ ╷ ╵ ╶─┴───┘ ┌─┘ ╵ ╵ │ ╵ └───┤ ╵ ╶─────┘ │ ╵ ╷ └───┴─┐ └─────┴─╴ │
    │     │           │       │       │           │   │       │           │
    └─────┴───────────┴───────┴───────┴───────────┴───┴───────┴───────────┘
    "
    );
}
