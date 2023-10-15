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

pub type BuildFunction = (fn(&mut maze::Maze), fn(&mut maze::Maze, speed::Speed));
pub type SolveFunction = (fn(maze::BoxMaze), fn(maze::BoxMaze, speed::Speed));

pub fn search_table<T>(arg: &str, table: &[(&'static str, T)]) -> Option<T>
where
    T: Clone,
{
    table
        .iter()
        .find(|(s, _)| *s == arg)
        .map(|(_, t)| t.clone())
}

pub const WALL_STYLES: [(&'static str, maze::MazeStyle); 6] = [
    ("sharp", (maze::MazeStyle::Sharp)),
    ("round", (maze::MazeStyle::Round)),
    ("doubles", (maze::MazeStyle::Doubles)),
    ("bold", (maze::MazeStyle::Bold)),
    ("contrast", (maze::MazeStyle::Contrast)),
    ("spikes", (maze::MazeStyle::Spikes)),
];
pub const BUILDERS: [(&'static str, BuildFunction); 9] = [
    ("arena", (arena::generate_maze, arena::animate_maze)),
    (
        "rdfs",
        (
            recursive_backtracker::generate_maze,
            recursive_backtracker::animate_maze,
        ),
    ),
    (
        "fractal",
        (
            recursive_subdivision::generate_maze,
            recursive_subdivision::animate_maze,
        ),
    ),
    ("prim", (prim::generate_maze, prim::animate_maze)),
    ("kruskal", (kruskal::generate_maze, kruskal::animate_maze)),
    ("eller", (eller::generate_maze, eller::animate_maze)),
    (
        "wilson",
        (wilson_carver::generate_maze, wilson_carver::animate_maze),
    ),
    (
        "wilson-walls",
        (wilson_adder::generate_maze, wilson_adder::animate_maze),
    ),
    ("grid", (grid::generate_maze, grid::animate_maze)),
];
pub const MODIFICATIONS: [(&'static str, BuildFunction); 2] = [
    ("cross", (modify::add_cross, modify::add_cross_animated)),
    ("x", (modify::add_x, modify::add_x_animated)),
];
pub const SOLVERS: [(&'static str, SolveFunction); 26] = [
    ("dfs-hunt", (dfs::hunt, dfs::animate_hunt)),
    ("dfs-gather", (dfs::gather, dfs::animate_gather)),
    ("dfs-corner", (dfs::corner, dfs::animate_corner)),
    ("darkdfs-hunt", (dfs::hunt, darkdfs::animate_hunt)),
    ("darkdfs-gather", (dfs::gather, darkdfs::animate_gather)),
    ("darkdfs-corner", (dfs::corner, darkdfs::animate_corner)),
    ("rdfs-hunt", (rdfs::hunt, rdfs::animate_hunt)),
    ("rdfs-gather", (rdfs::gather, rdfs::animate_gather)),
    ("rdfs-corner", (rdfs::corner, rdfs::animate_corner)),
    ("darkrdfs-hunt", (rdfs::hunt, darkrdfs::animate_hunt)),
    ("darkrdfs-gather", (rdfs::gather, darkrdfs::animate_gather)),
    ("darkrdfs-corner", (rdfs::corner, darkrdfs::animate_corner)),
    ("bfs-hunt", (bfs::hunt, bfs::animate_hunt)),
    ("bfs-gather", (bfs::gather, bfs::animate_gather)),
    ("bfs-corner", (bfs::corner, bfs::animate_corner)),
    ("darkbfs-hunt", (bfs::hunt, darkbfs::animate_hunt)),
    ("darkbfs-gather", (bfs::gather, darkbfs::animate_gather)),
    ("darkbfs-corner", (bfs::corner, darkbfs::animate_corner)),
    ("floodfs-hunt", (floodfs::hunt, floodfs::animate_hunt)),
    ("floodfs-gather", (floodfs::gather, floodfs::animate_gather)),
    ("floodfs-corner", (floodfs::corner, floodfs::animate_corner)),
    (
        "darkfloodfs-hunt",
        (floodfs::hunt, darkfloodfs::animate_hunt),
    ),
    (
        "darkfloodfs-gather",
        (floodfs::gather, darkfloodfs::animate_gather),
    ),
    (
        "darkfloodfs-corner",
        (floodfs::corner, darkfloodfs::animate_corner),
    ),
    ("runs", (runs::paint_run_lengths, runs::animate_run_lengths)),
    (
        "distance",
        (
            distance::paint_distance_from_center,
            distance::animate_distance_from_center,
        ),
    ),
];
pub const SPEEDS: [(&'static str, speed::Speed); 7] = [
    ("1", speed::Speed::Speed1),
    ("2", speed::Speed::Speed2),
    ("3", speed::Speed::Speed3),
    ("4", speed::Speed::Speed4),
    ("5", speed::Speed::Speed5),
    ("6", speed::Speed::Speed6),
    ("7", speed::Speed::Speed7),
];
