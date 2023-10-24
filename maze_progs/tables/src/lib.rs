pub use builders::arena;
pub use builders::eller;
pub use builders::grid;
pub use builders::hunt_kill;
pub use builders::kruskal;
pub use builders::modify;
pub use builders::prim;
pub use builders::recursive_backtracker;
pub use builders::recursive_subdivision;
pub use builders::wilson_adder;
pub use builders::wilson_carver;
pub use painters::distance;
pub use painters::rgb;
pub use painters::runs;
pub use solvers::bfs;
pub use solvers::darkbfs;
pub use solvers::darkdfs;
pub use solvers::darkfloodfs;
pub use solvers::darkrdfs;
pub use solvers::dfs;
pub use solvers::floodfs;
pub use solvers::rdfs;
pub use solvers::solve;

pub type BuildFunction = (fn(&mut maze::Maze), fn(&mut maze::Maze, speed::Speed));
pub type SolveFunction = (
    fn(solve::SolverMonitor),
    fn(solve::SolverMonitor, speed::Speed),
);

pub struct FlagArg<'a, 'b> {
    pub flag: &'a str,
    pub arg: &'b str,
}

#[derive(Clone, Copy)]
pub enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

#[derive(Clone, Copy)]
pub struct MazeRunner {
    pub args: maze::MazeArgs,
    pub build_view: ViewingMode,
    pub build_speed: speed::Speed,
    pub build: BuildFunction,
    pub modify: Option<BuildFunction>,
    pub solve_view: ViewingMode,
    pub solve_speed: speed::Speed,
    pub solve: SolveFunction,
}

impl MazeRunner {
    pub fn new() -> Self {
        Self {
            args: maze::MazeArgs {
                odd_rows: 33,
                odd_cols: 111,
                offset: maze::Offset::default(),
                style: maze::MazeStyle::Sharp,
            },
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

impl Default for MazeRunner {
    fn default() -> Self {
        Self::new()
    }
}

pub fn search_table<T>(arg: &str, table: &[(&str, T)]) -> Option<T>
where
    T: Clone,
{
    table
        .iter()
        .find(|(s, _)| *s == arg)
        .map(|(_, t)| t.clone())
}

pub fn load_desc(cur_builder: &BuildFunction) -> &'static str {
    match DESCRIPTIONS.iter().find(|(func, _)| func == cur_builder) {
        Some(&(_, desc)) => desc,
        None => "Coming Soon!",
    }
}

pub const FLAGS: [(&str, &str); 6] = [
    ("-b", "-b"),
    ("-m", "-m"),
    ("-s", "-s"),
    ("-w", "-w"),
    ("-sa", "-sa"),
    ("-ba", "-ba"),
];

pub const WALL_STYLES: [(&str, maze::MazeStyle); 8] = [
    ("mini", maze::MazeStyle::Mini),
    ("sharp", maze::MazeStyle::Sharp),
    ("round", maze::MazeStyle::Round),
    ("doubles", maze::MazeStyle::Doubles),
    ("bold", maze::MazeStyle::Bold),
    ("contrast", maze::MazeStyle::Contrast),
    ("half", maze::MazeStyle::Half),
    ("spikes", maze::MazeStyle::Spikes),
];

pub const BUILDERS: [(&str, BuildFunction); 10] = [
    ("arena", (arena::generate_maze, arena::animate_maze)),
    (
        "rdfs",
        (
            recursive_backtracker::generate_maze,
            recursive_backtracker::animate_maze,
        ),
    ),
    (
        "hunt-kill",
        (hunt_kill::generate_maze, hunt_kill::animate_maze),
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

pub const MODIFICATIONS: [(&str, BuildFunction); 2] = [
    ("cross", (modify::add_cross, modify::add_cross_animated)),
    ("x", (modify::add_x, modify::add_x_animated)),
];

pub const SOLVERS: [(&str, SolveFunction); 26] = [
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
    (
        "distance",
        (
            distance::paint_distance_from_center,
            distance::animate_distance_from_center,
        ),
    ),
    ("runs", (runs::paint_run_lengths, runs::animate_run_lengths)),
];

pub const SPEEDS: [(&str, speed::Speed); 7] = [
    ("1", speed::Speed::Speed1),
    ("2", speed::Speed::Speed2),
    ("3", speed::Speed::Speed3),
    ("4", speed::Speed::Speed4),
    ("5", speed::Speed::Speed5),
    ("6", speed::Speed::Speed6),
    ("7", speed::Speed::Speed7),
];

pub static DESCRIPTIONS: [(BuildFunction, &str); 10] = [
    (
        (
            builders::arena::generate_maze,
            builders::arena::animate_maze,
        ),
        include_str!("../../res/arena.txt"),
    ),
    (
        (
            builders::eller::generate_maze,
            builders::eller::animate_maze,
        ),
        include_str!("../../res/eller.txt"),
    ),
    (
        (builders::grid::generate_maze, builders::grid::animate_maze),
        include_str!("../../res/grid.txt"),
    ),
    (
        (
            builders::hunt_kill::generate_maze,
            builders::hunt_kill::animate_maze,
        ),
        include_str!("../../res/hunt_kill.txt"),
    ),
    (
        (
            builders::kruskal::generate_maze,
            builders::kruskal::animate_maze,
        ),
        include_str!("../../res/kruskal.txt"),
    ),
    (
        (builders::prim::generate_maze, builders::prim::animate_maze),
        include_str!("../../res/prim.txt"),
    ),
    (
        (
            builders::recursive_backtracker::generate_maze,
            builders::recursive_backtracker::animate_maze,
        ),
        include_str!("../../res/recursive_backtracker.txt"),
    ),
    (
        (
            builders::recursive_subdivision::generate_maze,
            builders::recursive_subdivision::animate_maze,
        ),
        include_str!("../../res/recursive_subdivision.txt"),
    ),
    (
        (
            builders::wilson_adder::generate_maze,
            builders::wilson_adder::animate_maze,
        ),
        include_str!("../../res/wilson_adder.txt"),
    ),
    (
        (
            builders::wilson_carver::generate_maze,
            builders::wilson_carver::animate_maze,
        ),
        include_str!("../../res/wilson_carver.txt"),
    ),
];
