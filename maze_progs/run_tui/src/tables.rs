pub use builders::arena;
pub use builders::eller;
pub use builders::grid;
pub use builders::kruskal;
pub use builders::modify;
pub use builders::prim;
pub use builders::recursive_backtracker;
pub use builders::recursive_subdivision;
pub use builders::wilson_adder;
pub use builders::wilson_carver;
pub use painters::distance;
pub use painters::runs;
pub use solvers::bfs;
pub use solvers::darkbfs;
pub use solvers::darkdfs;
pub use solvers::darkfloodfs;
pub use solvers::darkrdfs;
pub use solvers::dfs;
pub use solvers::floodfs;
pub use solvers::rdfs;

use std::collections::{HashMap, HashSet};

type BuildFunction = (fn(&mut maze::Maze), fn(&mut maze::Maze, speed::Speed));
type SolveFunction = (fn(maze::BoxMaze), fn(maze::BoxMaze, speed::Speed));

pub struct LookupTables {
    pub arg_flags: HashSet<String>,
    pub build_table: HashMap<String, BuildFunction>,
    pub mod_table: HashMap<String, BuildFunction>,
    pub solve_table: HashMap<String, SolveFunction>,
    pub style_table: HashMap<String, maze::MazeStyle>,
    pub animation_table: HashMap<String, speed::Speed>,
}

pub fn load_function_tables() -> LookupTables {
    LookupTables {
        arg_flags: HashSet::from([
            String::from("-r"),
            String::from("-c"),
            String::from("-b"),
            String::from("-s"),
            String::from("-h"),
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
            (
                String::from("darkdfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    darkdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    darkdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    darkdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    darkfloodfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    darkfloodfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    darkfloodfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    darkrdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    darkrdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    darkrdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    darkbfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    darkbfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    darkbfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
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
    }
}
