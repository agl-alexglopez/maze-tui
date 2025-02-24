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
pub struct HistoryRunner {
    pub args: maze::MazeArgs,
    pub build: BuildHistoryType,
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
            build: BuildHistoryType::RecursiveBacktracker(recursive_backtracker::generate_history),
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

pub fn search_table<T>(arg: &str, table: &[(&str, T)]) -> Option<T>
where
    T: Clone,
{
    table
        .iter()
        .find(|(s, _)| *s == arg)
        .map(|(_, t)| t.clone())
}

pub fn load_info(cur_builder: &BuildHistoryType) -> &'static str {
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

#[derive(Clone)]
pub struct BuildHistoryEntry(pub BuildHistoryFunction);
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuildHistoryType {
    Arena(BuildHistoryFunction),
    RecursiveBacktracker(BuildHistoryFunction),
    HuntKill(BuildHistoryFunction),
    RecursiveSubdivision(BuildHistoryFunction),
    Prim(BuildHistoryFunction),
    Kruskal(BuildHistoryFunction),
    Eller(BuildHistoryFunction),
    WilsonCarver(BuildHistoryFunction),
    WilsonAdder(BuildHistoryFunction),
    Grid(BuildHistoryFunction),
}
impl BuildHistoryType {
    pub fn function(&self) -> fn(std::sync::Arc<std::sync::Mutex<monitor::Monitor>>) {
        match self {
            BuildHistoryType::Arena(f) => *f,
            BuildHistoryType::RecursiveBacktracker(f) => *f,
            BuildHistoryType::HuntKill(f) => *f,
            BuildHistoryType::RecursiveSubdivision(f) => *f,
            BuildHistoryType::Prim(f) => *f,
            BuildHistoryType::Kruskal(f) => *f,
            BuildHistoryType::Eller(f) => *f,
            BuildHistoryType::WilsonCarver(f) => *f,
            BuildHistoryType::WilsonAdder(f) => *f,
            BuildHistoryType::Grid(f) => *f,
        }
    }
}
///
/// History and playback specific tables
///
pub const HISTORY_BUILDERS: [(&str, BuildHistoryType); 10] = [
    ("arena", BuildHistoryType::Arena(arena::generate_history)),
    (
        "rdfs",
        BuildHistoryType::RecursiveBacktracker(recursive_backtracker::generate_history),
    ),
    (
        "hunt-kill",
        BuildHistoryType::HuntKill(hunt_kill::generate_history),
    ),
    (
        "fractal",
        BuildHistoryType::RecursiveSubdivision(recursive_subdivision::generate_history),
    ),
    ("prim", BuildHistoryType::Prim(prim::generate_history)),
    (
        "kruskal",
        BuildHistoryType::Kruskal(kruskal::generate_history),
    ),
    ("eller", BuildHistoryType::Eller(eller::generate_history)),
    (
        "wilson",
        BuildHistoryType::WilsonCarver(wilson_carver::generate_history),
    ),
    (
        "wilson-walls",
        BuildHistoryType::WilsonAdder(wilson_adder::generate_history),
    ),
    ("grid", BuildHistoryType::Grid(grid::generate_history)),
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

pub static DESCRIPTIONS: [(BuildHistoryType, &str); 10] = [
    (
        BuildHistoryType::Arena(builders::arena::generate_history),
        include_str!("../../res/arena.txt"),
    ),
    (
        BuildHistoryType::Eller(builders::eller::generate_history),
        include_str!("../../res/eller.txt"),
    ),
    (
        BuildHistoryType::Grid(builders::grid::generate_history),
        include_str!("../../res/grid.txt"),
    ),
    (
        BuildHistoryType::HuntKill(builders::hunt_kill::generate_history),
        include_str!("../../res/hunt_kill.txt"),
    ),
    (
        BuildHistoryType::Kruskal(builders::kruskal::generate_history),
        include_str!("../../res/kruskal.txt"),
    ),
    (
        BuildHistoryType::Prim(builders::prim::generate_history),
        include_str!("../../res/prim.txt"),
    ),
    (
        BuildHistoryType::RecursiveBacktracker(builders::recursive_backtracker::generate_history),
        include_str!("../../res/recursive_backtracker.txt"),
    ),
    (
        BuildHistoryType::RecursiveSubdivision(builders::recursive_subdivision::generate_history),
        include_str!("../../res/recursive_subdivision.txt"),
    ),
    (
        BuildHistoryType::WilsonAdder(builders::wilson_adder::generate_history),
        include_str!("../../res/wilson_adder.txt"),
    ),
    (
        BuildHistoryType::WilsonCarver(builders::wilson_carver::generate_history),
        include_str!("../../res/wilson_carver.txt"),
    ),
];
