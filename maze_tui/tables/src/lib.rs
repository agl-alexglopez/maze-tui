use rand::seq::SliceRandom;

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
    pub modify: Option<ModificationHistoryType>,
    pub solve: SolveHistoryType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuildHistoryType {
    Arena = 0,
    RecursiveBacktracker,
    HuntKill,
    RecursiveSubdivision,
    Prim,
    Kruskal,
    Eller,
    WilsonCarver,
    WilsonAdder,
    Grid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SolveHistoryType {
    DfsHunt = 0,
    DfsGather,
    DfsCorner,
    RdfsHunt,
    RdfsGather,
    RdfsCorner,
    BfsHunt,
    BfsGather,
    BfsCorner,
    FdfsHunt,
    FdfsGather,
    FdfsCorner,
    Distance,
    Runs,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModificationHistoryType {
    Cross = 0,
    X,
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
            build: BuildHistoryType::RecursiveBacktracker,
            modify: None,
            solve: SolveHistoryType::DfsHunt,
        }
    }
}

impl Default for HistoryRunner {
    fn default() -> Self {
        Self::new()
    }
}

fn search_table<T>(arg: &str, table: &[(&str, T)]) -> Option<T>
where
    T: Clone,
{
    table
        .iter()
        .find(|(s, _)| *s == arg)
        .map(|(_, t)| t.clone())
}

pub fn match_flag(arg: &str) -> Option<&str> {
    search_table(arg, &FLAGS)
}

pub fn match_builder(arg: &str) -> Option<BuildHistoryType> {
    search_table(arg, &HISTORY_BUILDERS)
}

pub fn match_modifier(arg: &str) -> Option<ModificationHistoryType> {
    search_table(arg, &HISTORY_MODIFICATIONS)
}

pub fn match_solver(arg: &str) -> Option<SolveHistoryType> {
    search_table(arg, &HISTORY_SOLVERS)
}

pub fn match_walls(arg: &str) -> Option<maze::MazeStyle> {
    search_table(arg, &WALL_STYLES)
}

impl BuildHistoryType {
    fn index(&self) -> usize {
        match self {
            BuildHistoryType::Arena => 0,
            BuildHistoryType::RecursiveBacktracker => 1,
            BuildHistoryType::HuntKill => 2,
            BuildHistoryType::RecursiveSubdivision => 3,
            BuildHistoryType::Prim => 4,
            BuildHistoryType::Kruskal => 5,
            BuildHistoryType::Eller => 6,
            BuildHistoryType::WilsonCarver => 7,
            BuildHistoryType::WilsonAdder => 8,
            BuildHistoryType::Grid => 9,
        }
    }

    pub fn get_fn(&self) -> BuildHistoryFunction {
        BUILD_FN_TABLE[self.index()]
    }

    pub fn get_description(&self) -> &str {
        BUILD_DESCRIPTIONS_TABLE[self.index()]
    }

    pub fn get_random(rng: &mut rand::rngs::ThreadRng) -> BuildHistoryType {
        *ALL_BUILDER_TYPES
            .choose(rng)
            .expect("cannot choose random builder")
    }
}

impl SolveHistoryType {
    fn index(&self) -> usize {
        match self {
            SolveHistoryType::DfsHunt => 0,
            SolveHistoryType::DfsGather => 1,
            SolveHistoryType::DfsCorner => 2,
            SolveHistoryType::RdfsHunt => 3,
            SolveHistoryType::RdfsGather => 4,
            SolveHistoryType::RdfsCorner => 5,
            SolveHistoryType::BfsHunt => 6,
            SolveHistoryType::BfsGather => 7,
            SolveHistoryType::BfsCorner => 8,
            SolveHistoryType::FdfsHunt => 9,
            SolveHistoryType::FdfsGather => 10,
            SolveHistoryType::FdfsCorner => 11,
            SolveHistoryType::Distance => 12,
            SolveHistoryType::Runs => 13,
        }
    }

    pub fn get_fn(&self) -> SolveHistoryFunction {
        SOLVE_FN_TABLE[self.index()]
    }

    pub fn get_random(rng: &mut rand::rngs::ThreadRng) -> SolveHistoryType {
        *ALL_SOLVER_TYPES
            .choose(rng)
            .expect("cannot choose random solver")
    }
}

impl ModificationHistoryType {
    fn index(&self) -> usize {
        match self {
            ModificationHistoryType::Cross => 0,
            ModificationHistoryType::X => 1,
        }
    }

    pub fn get_fn(&self) -> BuildHistoryFunction {
        MODIFICATION_FN_TABLE[self.index()]
    }

    pub fn get_random(rng: &mut rand::rngs::ThreadRng) -> ModificationHistoryType {
        *ALL_MODIFICATION_TYPES
            .choose(rng)
            .expect("cannot modify the maze")
    }
}

static FLAGS: [(&str, &str); 6] = [
    ("-b", "-b"),
    ("-m", "-m"),
    ("-s", "-s"),
    ("-w", "-w"),
    ("-sa", "-sa"),
    ("-ba", "-ba"),
];

static WALL_STYLES: [(&str, maze::MazeStyle); 8] = [
    ("mini", maze::MazeStyle::Mini),
    ("sharp", maze::MazeStyle::Sharp),
    ("round", maze::MazeStyle::Round),
    ("doubles", maze::MazeStyle::Doubles),
    ("bold", maze::MazeStyle::Bold),
    ("contrast", maze::MazeStyle::Contrast),
    ("half", maze::MazeStyle::Half),
    ("spikes", maze::MazeStyle::Spikes),
];

static HISTORY_BUILDERS: [(&str, BuildHistoryType); 10] = [
    ("arena", BuildHistoryType::Arena),
    ("rdfs", BuildHistoryType::RecursiveBacktracker),
    ("hunt-kill", BuildHistoryType::HuntKill),
    ("fractal", BuildHistoryType::RecursiveSubdivision),
    ("prim", BuildHistoryType::Prim),
    ("kruskal", BuildHistoryType::Kruskal),
    ("eller", BuildHistoryType::Eller),
    ("wilson", BuildHistoryType::WilsonCarver),
    ("wilson-walls", BuildHistoryType::WilsonAdder),
    ("grid", BuildHistoryType::Grid),
];

static HISTORY_MODIFICATIONS: [(&str, ModificationHistoryType); 2] = [
    ("cross", ModificationHistoryType::Cross),
    ("x", ModificationHistoryType::X),
];

static HISTORY_SOLVERS: [(&str, SolveHistoryType); 14] = [
    ("dfs-hunt", SolveHistoryType::DfsHunt),
    ("dfs-gather", SolveHistoryType::DfsGather),
    ("dfs-corner", SolveHistoryType::DfsCorner),
    ("rdfs-hunt", SolveHistoryType::RdfsHunt),
    ("rdfs-gather", SolveHistoryType::RdfsGather),
    ("rdfs-corner", SolveHistoryType::RdfsCorner),
    ("bfs-hunt", SolveHistoryType::BfsHunt),
    ("bfs-gather", SolveHistoryType::BfsGather),
    ("bfs-corner", SolveHistoryType::BfsCorner),
    ("floodfs-hunt", SolveHistoryType::FdfsHunt),
    ("floodfs-gather", SolveHistoryType::FdfsGather),
    ("floodfs-corner", SolveHistoryType::FdfsCorner),
    ("distance", SolveHistoryType::Distance),
    ("runs", SolveHistoryType::Runs),
];

static BUILD_FN_TABLE: [BuildHistoryFunction; 10] = [
    arena::generate_history,
    recursive_backtracker::generate_history,
    hunt_kill::generate_history,
    recursive_subdivision::generate_history,
    prim::generate_history,
    kruskal::generate_history,
    eller::generate_history,
    wilson_carver::generate_history,
    wilson_adder::generate_history,
    grid::generate_history,
];

static MODIFICATION_FN_TABLE: [BuildHistoryFunction; 2] =
    [modify::add_cross_history, modify::add_x_history];

static SOLVE_FN_TABLE: [SolveHistoryFunction; 14] = [
    dfs::hunt_history,
    dfs::gather_history,
    dfs::corner_history,
    rdfs::hunt_history,
    rdfs::gather_history,
    rdfs::corner_history,
    bfs::hunt_history,
    bfs::gather_history,
    bfs::corner_history,
    floodfs::hunt_history,
    floodfs::gather_history,
    floodfs::corner_history,
    distance::paint_distance_from_center_history,
    runs::paint_run_lengths_history,
];

static ALL_BUILDER_TYPES: [BuildHistoryType; 10] = [
    BuildHistoryType::Arena,
    BuildHistoryType::RecursiveBacktracker,
    BuildHistoryType::HuntKill,
    BuildHistoryType::RecursiveSubdivision,
    BuildHistoryType::Prim,
    BuildHistoryType::Kruskal,
    BuildHistoryType::Eller,
    BuildHistoryType::WilsonCarver,
    BuildHistoryType::WilsonAdder,
    BuildHistoryType::Grid,
];

static BUILD_DESCRIPTIONS_TABLE: [&str; 10] = [
    include_str!("../../res/arena.txt"),
    include_str!("../../res/recursive_backtracker.txt"),
    include_str!("../../res/hunt_kill.txt"),
    include_str!("../../res/recursive_subdivision.txt"),
    include_str!("../../res/prim.txt"),
    include_str!("../../res/kruskal.txt"),
    include_str!("../../res/eller.txt"),
    include_str!("../../res/wilson_carver.txt"),
    include_str!("../../res/wilson_adder.txt"),
    include_str!("../../res/grid.txt"),
];

static ALL_MODIFICATION_TYPES: [ModificationHistoryType; 2] =
    [ModificationHistoryType::Cross, ModificationHistoryType::X];

static ALL_SOLVER_TYPES: [SolveHistoryType; 14] = [
    SolveHistoryType::DfsHunt,
    SolveHistoryType::DfsGather,
    SolveHistoryType::DfsCorner,
    SolveHistoryType::RdfsHunt,
    SolveHistoryType::RdfsGather,
    SolveHistoryType::RdfsCorner,
    SolveHistoryType::BfsHunt,
    SolveHistoryType::BfsGather,
    SolveHistoryType::BfsCorner,
    SolveHistoryType::FdfsHunt,
    SolveHistoryType::FdfsGather,
    SolveHistoryType::FdfsCorner,
    SolveHistoryType::Distance,
    SolveHistoryType::Runs,
];
