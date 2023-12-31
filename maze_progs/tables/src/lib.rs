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
pub use monitor;
pub use painters::distance;
pub use painters::rgb;
pub use painters::runs;
pub use solvers::bfs;
pub use solvers::dfs;
pub use solvers::floodfs;
pub use solvers::rdfs;
pub use solvers::solve;

pub type BuildCursorFunction = (
    fn(monitor::MazeReceiver),
    fn(monitor::MazeReceiver, speed::Speed),
);
pub type SolveCursorFunction = (
    fn(monitor::MazeReceiver),
    fn(monitor::MazeReceiver, speed::Speed),
);
pub type BuildHistoryFunction = fn(monitor::MazeMonitor);
pub type SolveHistoryFunction = fn(monitor::MazeMonitor);

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
pub struct CursorRunner {
    pub args: maze::MazeArgs,
    pub build_view: ViewingMode,
    pub build_speed: speed::Speed,
    pub build: BuildCursorFunction,
    pub modify: Option<BuildCursorFunction>,
    pub solve_view: ViewingMode,
    pub solve_speed: speed::Speed,
    pub solve: SolveCursorFunction,
}

#[derive(Clone, Copy)]
pub struct HistoryRunner {
    pub args: maze::MazeArgs,
    pub build: BuildHistoryFunction,
    pub modify: Option<BuildHistoryFunction>,
    pub solve: SolveHistoryFunction,
}

impl HistoryRunner {
    pub fn new() -> Self {
        Self {
            args: maze::MazeArgs {
                odd_rows: 33,
                odd_cols: 111,
                offset: maze::Offset::default(),
                style: maze::MazeStyle::Sharp,
            },
            build: recursive_backtracker::generate_history,
            modify: None,
            solve: dfs::hunt_history,
        }
    }
}

impl Default for HistoryRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorRunner {
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

impl Default for CursorRunner {
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

pub fn load_info(cur_builder: &BuildHistoryFunction) -> &'static str {
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

///
/// History and playback specific tables
///

pub const HISTORY_BUILDERS: [(&str, BuildHistoryFunction); 10] = [
    ("arena", arena::generate_history),
    ("rdfs", recursive_backtracker::generate_history),
    ("hunt-kill", hunt_kill::generate_history),
    ("fractal", recursive_subdivision::generate_history),
    ("prim", prim::generate_history),
    ("kruskal", kruskal::generate_history),
    ("eller", eller::generate_history),
    ("wilson", wilson_carver::generate_history),
    ("wilson-walls", wilson_adder::generate_history),
    ("grid", grid::generate_history),
];

pub const HISTORY_MODIFICATIONS: [(&str, BuildHistoryFunction); 2] = [
    ("cross", modify::add_cross_history),
    ("x", modify::add_x_history),
];

pub const HISTORY_SOLVERS: [(&str, SolveHistoryFunction); 14] = [
    ("dfs-hunt", dfs::hunt_history),
    ("dfs-gather", dfs::gather_history),
    ("dfs-corner", dfs::corner_history),
    ("rdfs-hunt", rdfs::hunt_history),
    ("rdfs-gather", rdfs::gather_history),
    ("rdfs-corner", rdfs::corner_history),
    ("bfs-hunt", bfs::hunt_history),
    ("bfs-gather", bfs::gather_history),
    ("bfs-corner", bfs::corner_history),
    ("floodfs-hunt", floodfs::hunt_history),
    ("floodfs-gather", floodfs::gather_history),
    ("floodfs-corner", floodfs::corner_history),
    ("distance", distance::paint_distance_from_center_history),
    ("runs", runs::paint_run_lengths_history),
];

pub static DESCRIPTIONS: [(BuildHistoryFunction, &str); 10] = [
    (
        builders::arena::generate_history,
        include_str!("../../res/arena.txt"),
    ),
    (
        builders::eller::generate_history,
        include_str!("../../res/eller.txt"),
    ),
    (
        builders::grid::generate_history,
        include_str!("../../res/grid.txt"),
    ),
    (
        builders::hunt_kill::generate_history,
        include_str!("../../res/hunt_kill.txt"),
    ),
    (
        builders::kruskal::generate_history,
        include_str!("../../res/kruskal.txt"),
    ),
    (
        builders::prim::generate_history,
        include_str!("../../res/prim.txt"),
    ),
    (
        builders::recursive_backtracker::generate_history,
        include_str!("../../res/recursive_backtracker.txt"),
    ),
    (
        builders::recursive_subdivision::generate_history,
        include_str!("../../res/recursive_subdivision.txt"),
    ),
    (
        builders::wilson_adder::generate_history,
        include_str!("../../res/wilson_adder.txt"),
    ),
    (
        builders::wilson_carver::generate_history,
        include_str!("../../res/wilson_carver.txt"),
    ),
];

///
/// Cursor animation specific lookups
///

pub const SPEEDS: [(&str, speed::Speed); 7] = [
    ("1", speed::Speed::Speed1),
    ("2", speed::Speed::Speed2),
    ("3", speed::Speed::Speed3),
    ("4", speed::Speed::Speed4),
    ("5", speed::Speed::Speed5),
    ("6", speed::Speed::Speed6),
    ("7", speed::Speed::Speed7),
];

pub const CURSOR_BUILDERS: [(&str, BuildCursorFunction); 10] = [
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

pub const CURSOR_MODIFICATIONS: [(&str, BuildCursorFunction); 2] = [
    ("cross", (modify::add_cross, modify::add_cross_animated)),
    ("x", (modify::add_x, modify::add_x_animated)),
];

pub const CURSOR_SOLVERS: [(&str, SolveCursorFunction); 14] = [
    ("dfs-hunt", (dfs::hunt, dfs::animate_hunt)),
    ("dfs-gather", (dfs::gather, dfs::animate_gather)),
    ("dfs-corner", (dfs::corner, dfs::animate_corner)),
    ("rdfs-hunt", (rdfs::hunt, rdfs::animate_hunt)),
    ("rdfs-gather", (rdfs::gather, rdfs::animate_gather)),
    ("rdfs-corner", (rdfs::corner, rdfs::animate_corner)),
    ("bfs-hunt", (bfs::hunt, bfs::animate_hunt)),
    ("bfs-gather", (bfs::gather, bfs::animate_gather)),
    ("bfs-corner", (bfs::corner, bfs::animate_corner)),
    ("floodfs-hunt", (floodfs::hunt, floodfs::animate_hunt)),
    ("floodfs-gather", (floodfs::gather, floodfs::animate_gather)),
    ("floodfs-corner", (floodfs::corner, floodfs::animate_corner)),
    (
        "distance",
        (
            distance::paint_distance_from_center,
            distance::animate_distance_from_center,
        ),
    ),
    ("runs", (runs::paint_run_lengths, runs::animate_run_lengths)),
];
