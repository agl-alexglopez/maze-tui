use builders::arena;
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
use rand::{
    distributions::{Bernoulli, Distribution},
    seq::SliceRandom,
    thread_rng,
};
use solvers::bfs;
use solvers::darkbfs;
use solvers::darkdfs;
use solvers::darkfloodfs;
use solvers::darkrdfs;
use solvers::dfs;
use solvers::floodfs;
use solvers::rdfs;

type BuildAnimation = fn(&mut maze::Maze, speed::Speed);
type SolveAnimation = fn(maze::BoxMaze, speed::Speed);

pub fn rand(mut args: maze::MazeArgs) {
    let mut rng = thread_rng();
    let modification_probability = Bernoulli::new(0.2);
    match WALL_STYLES.choose(&mut rng) {
        Some(&style) => args.style = style,
        None => print::maze_panic!("Styles not set for loop, broken"),
    }
    let mut maze = maze::Maze::new(args);
    let build_speed = match SPEEDS.choose(&mut rng) {
        Some(&speed) => speed,
        None => print::maze_panic!("Build speed array empty."),
    };
    let solve_speed = match SPEEDS.choose(&mut rng) {
        Some(&speed) => speed,
        None => print::maze_panic!("Solve speed array empty."),
    };
    let build_algo = match BUILDERS.choose(&mut rng) {
        Some(&algo) => algo,
        None => print::maze_panic!("Build algorithm array empty."),
    };
    let solve_algo = match SOLVERS.choose(&mut rng) {
        Some(&algo) => algo,
        None => print::maze_panic!("Solve algorithm array empty."),
    };
    build_algo(&mut maze, build_speed);
    if modification_probability
        .expect("Bernoulli innefective")
        .sample(&mut rng)
    {
        match MODIFICATIONS.choose(&mut rng) {
            Some(modder) => {
                modder(&mut maze, build_speed);
            }
            None => print::maze_panic!("Empty modification table."),
        }
    }
    print::set_cursor_position(maze::Point::default(), args.offset);
    solve_algo(maze, solve_speed);
}

const WALL_STYLES: [maze::MazeStyle; 6] = [
    maze::MazeStyle::Sharp,
    maze::MazeStyle::Round,
    maze::MazeStyle::Doubles,
    maze::MazeStyle::Bold,
    maze::MazeStyle::Contrast,
    maze::MazeStyle::Spikes,
];
const BUILDERS: [BuildAnimation; 1] = [
    arena::animate_maze,
    // recursive_backtracker::animate_maze,
    // recursive_subdivision::animate_maze,
    // prim::animate_maze,
    // kruskal::animate_maze,
    // eller::animate_maze,
    // wilson_carver::animate_maze,
    // wilson_adder::animate_maze,
    // grid::animate_maze,
];
const MODIFICATIONS: [BuildAnimation; 2] = [modify::add_cross_animated, modify::add_x_animated];
const SOLVERS: [SolveAnimation; 26] = [
    dfs::animate_hunt,
    dfs::animate_gather,
    dfs::animate_corner,
    darkdfs::animate_hunt,
    darkdfs::animate_gather,
    darkdfs::animate_corner,
    rdfs::animate_hunt,
    rdfs::animate_gather,
    rdfs::animate_corner,
    darkrdfs::animate_hunt,
    darkrdfs::animate_gather,
    darkrdfs::animate_corner,
    bfs::animate_hunt,
    bfs::animate_gather,
    bfs::animate_corner,
    darkbfs::animate_hunt,
    darkbfs::animate_gather,
    darkbfs::animate_corner,
    floodfs::animate_hunt,
    floodfs::animate_gather,
    floodfs::animate_corner,
    darkfloodfs::animate_hunt,
    darkfloodfs::animate_gather,
    darkfloodfs::animate_corner,
    runs::animate_run_lengths,
    distance::animate_distance_from_center,
];
const SPEEDS: [speed::Speed; 4] = [
    speed::Speed::Speed3,
    speed::Speed::Speed4,
    speed::Speed::Speed5,
    speed::Speed::Speed6,
];
