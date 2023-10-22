use builders::build::{self, flush_grid};
use solvers::solve;
use tables;

use std::env;

fn main() {
    // RAII approach to cursor hiding. Call hide and on scope drop it unhides, no call needed.
    let invisible = print::InvisibleCursor::new();
    invisible.hide();
    ctrlc::set_handler(move || {
        //print::clear_screen();
        print::set_cursor_position(maze::Point { row: 50, col: 111 }, maze::Offset::default());
        print::unhide_cursor_on_process_exit();
        std::process::exit(0);
    })
    .expect("Could not set quit handler.");

    let mut run = tables::MazeRunner::new();

    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in env::args().skip(1) {
        if process_current {
            match set_arg(
                &mut run,
                &tables::FlagArg {
                    flag: prev_flag,
                    arg: &a,
                },
            ) {
                Ok(_) => {}
                Err(msg) => print::maze_panic!("{}", msg),
            }
            process_current = false;
            continue;
        }
        match tables::search_table(&a, &tables::FLAGS) {
            Some(flag) => {
                process_current = true;
                prev_flag = flag;
            }
            None => match &*a {
                "-r" => {
                    process_current = true;
                    prev_flag = "-r";
                }
                "-c" => {
                    process_current = true;
                    prev_flag = "-c";
                }
                _ => {
                    quit(&err_string(&tables::FlagArg {
                        flag: &a,
                        arg: "[NONE]",
                    }));
                }
            },
        }
    }
    if process_current {
        quit(&err_string(&tables::FlagArg {
            flag: &prev_flag,
            arg: "[NONE]",
        }));
    }

    let mut maze = maze::Maze::new(run.args);

    print::clear_screen();
    build::print_overlap_key(&maze);
    match run.build_view {
        tables::ViewingMode::StaticImage => {
            run.build.0(&mut maze);
            flush_grid(&maze);
            if let Some((static_mod, _)) = run.modify {
                static_mod(&mut maze)
            }
        }
        tables::ViewingMode::AnimatedPlayback => {
            run.build.1(&mut maze, run.build_speed);
            if let Some((_, animate_mod)) = run.modify {
                animate_mod(&mut maze, run.build_speed)
            }
        }
    }

    // Ensure a smooth transition from build to solve with no flashing.
    print::set_cursor_position(maze::Point { row: 50, col: 0 }, maze::Offset::default());

    // Commented out for testing builders only right now.

    // let monitor = solve::Solver::new(maze);

    // match run.solve_view {
    //     tables::ViewingMode::StaticImage => {
    //         run.solve.0(monitor.clone());
    //     }
    //     tables::ViewingMode::AnimatedPlayback => run.solve.1(monitor.clone(), run.solve_speed),
    // }

    // if let Ok(lk) = monitor.clone().lock() {
    //     print::set_cursor_position(
    //         maze::Point {
    //             row: lk.maze.row_size() + 2,
    //             col: 0,
    //         },
    //         maze::Offset::default(),
    //     );
    // }
}

fn set_arg(run: &mut tables::MazeRunner, args: &tables::FlagArg) -> Result<(), String> {
    match args.flag {
        "-h" => return Err("".to_string()),
        "-r" => {
            run.args.odd_rows = set_dimension(args);
            Ok(())
        }
        "-c" => {
            run.args.odd_cols = set_dimension(args);
            Ok(())
        }
        "-b" => tables::search_table(args.arg, &tables::BUILDERS)
            .map(|func_pair| run.build = func_pair)
            .ok_or(err_string(args)),
        "-m" => tables::search_table(args.arg, &tables::MODIFICATIONS)
            .map(|mod_tuple| run.modify = Some(mod_tuple))
            .ok_or(err_string(args)),
        "-s" => tables::search_table(args.arg, &tables::SOLVERS)
            .map(|solve_tuple| run.solve = solve_tuple)
            .ok_or(err_string(args)),
        "-w" => tables::search_table(args.arg, &tables::WALL_STYLES)
            .map(|wall_style| run.args.style = wall_style)
            .ok_or(err_string(args)),
        "-ba" => match tables::search_table(args.arg, &tables::SPEEDS) {
            Some(speed) => {
                run.build_speed = speed;
                run.build_view = tables::ViewingMode::AnimatedPlayback;
                Ok(())
            }
            None => Err(err_string(args)),
        },
        "-sa" => match tables::search_table(args.arg, &tables::SPEEDS) {
            Some(speed) => {
                run.solve_speed = speed;
                run.solve_view = tables::ViewingMode::AnimatedPlayback;
                Ok(())
            }
            None => Err(err_string(args)),
        },
        _ => Err(err_string(args)),
    }
}

pub fn err_string(args: &tables::FlagArg) -> String {
    String::from(format!(
        "invalid flag[{}] arg[{}] combo",
        args.flag, args.arg
    ))
}

fn set_dimension(pairs: &tables::FlagArg) -> i32 {
    match pairs.arg.parse::<i32>() {
        Ok(num) => {
            if num < 7 {
                quit("Invalid row or column dimension");
                std::process::exit(1);
            }
            num
        }
        Err(_) => {
            quit("Invalid row or column dimension");
            std::process::exit(1);
        }
    }
}

fn quit(msg: &str) {
    println!("{}", msg);
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
    │ │ │ │ │   dark[solver]-[game] - A mystery...│   │ │   │   │ │     │ │
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
